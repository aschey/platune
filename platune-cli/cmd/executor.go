package cmd

import (
	"fmt"
	"io/fs"
	"os"
	"path"
	"strings"

	"github.com/aschey/go-prompt"
	"github.com/aschey/platune/cli/v2/internal"
	"github.com/aschey/platune/cli/v2/internal/mode"
	management_v1 "github.com/aschey/platune/client/management_v1"
	"github.com/aschey/platune/client/player_v1"
)

func (state *cmdState) executor(in string, selected *prompt.Suggest, suggestions []prompt.Suggest) {
	isSetQueueMode := state.mode.First() == mode.SetQueueMode
	if selected == nil {
		// User did not explicitly choose a result
		// See if we can find a match instead
		text := strings.TrimSpace(strings.ToLower(in))
		if strings.HasPrefix(text, addQueueCmdText) && state.mode.Current() == mode.NormalMode {
			cmds := strings.SplitN(text, " ", 2)
			text = strings.TrimSpace(cmds[len(cmds)-1])
		}
		for _, suggestion := range suggestions {
			if strings.ToLower(suggestion.Text) == text {
				selected = &suggestion
				break
			}
		}
	}
	if state.mode.Current() != mode.NormalMode {
		state.executeMode(in, selected)
	} else {
		cmds := strings.SplitN(in, " ", 2)
		if len(cmds) == 0 {
			return
		}

		state.executeCmd(cmds, selected)
	}

	if state.mode.First() == mode.NormalMode && len(state.currentQueue) > 0 {
		if isSetQueueMode && strings.TrimSpace(in) == "" {
			state.client.SetQueueFromSearchResults(state.currentQueue, true)
			state.currentQueue = []*management_v1.LookupEntry{}
		} else if !isSetQueueMode {
			state.client.AddSearchResultsToQueue(state.currentQueue, true)
			state.currentQueue = []*management_v1.LookupEntry{}
		}
	}
}

func (state *cmdState) executeMode(in string, selected *prompt.Suggest) {
	switch state.mode.Current() {
	case mode.SetQueueMode:
		if strings.TrimSpace(in) == "" {
			state.mode = mode.NewDefaultMode()
		} else {
			pathInput, err := getPathInput(in, selected)
			if err != nil {
				fmt.Println(err)
			} else if pathInput == "" {
				state.executeEntryType(selected, mode.SetQueueMode)
			} else {
				state.currentQueue = append(state.currentQueue, &management_v1.LookupEntry{Path: in})
			}
		}

	case mode.AlbumMode:
		if selected == nil || state.checkSpecialOptions(selected) {
			return
		}
		state.mode.Set(mode.SongMode)
		newResults := []*management_v1.LookupEntry{}
		for _, r := range state.lookupResult {
			album := r.Album
			if strings.TrimSpace(r.Album) == "" {
				album = "[Untitled]"
			}
			if album == in {
				newResults = append(newResults, r)
			}
		}
		state.lookupResult = newResults
		return
	case mode.SongMode:
		if selected == nil || state.checkSpecialOptions(selected) {
			return
		}
		lookupResponse := selected.Metadata.(*management_v1.LookupEntry)
		state.mode.Reset()
		state.currentQueue = append(state.currentQueue, lookupResponse)

		return
	}
}

func (state *cmdState) executeCmd(cmds []string, selected *prompt.Suggest) {
	rest := strings.Join(cmds[1:], " ")
	switch cmds[0] {
	case setQueueCmdText:
		fmt.Println("Enter file paths or urls to add to the queue.")
		fmt.Println("Enter a blank line when done.")
		state.mode = mode.NewMode(mode.SetQueueMode)
		return
	case addQueueCmdText:
		if len(cmds) < 2 {
			fmt.Printf("Usage: %s <path or url>\n", addQueueCmdText)
			return
		}

		pathInput, err := getPathInput(rest, selected)
		if err != nil {
			fmt.Println(err)
			return
		}

		if pathInput == "" {
			state.executeEntryType(selected, mode.NormalMode)
			return
		}

		state.client.AddToQueue([]*player_v1.Track{{Url: pathInput}}, true)
	case seekCmdText:
		if len(cmds) < 2 {
			fmt.Println("Usage: seek [hh]:[mm]:ss")
			return
		}
		state.client.Seek(cmds[1])
	case setVolumeCmdText:
		if len(cmds) < 2 {
			fmt.Println("Usage: " + setVolumeUsage)
		}
		runSetVolume(state.client, cmds[1:])
	case pauseCmdText:
		state.client.Pause()
	case resumeCmdText:
		state.client.Resume()
	case stopCmdText:
		state.client.Stop()
	case nextCmdText:
		state.client.Next()
	case previousCmdText:
		state.client.Previous()
	case syncCmdText:
		syncProgress(state.client, state.deleted)
		fmt.Println()
	case getAllFoldersCmdText:
		state.client.GetAllFolders()
	case addFolderCmdText:
		if len(cmds) < 2 {
			fmt.Printf("Usage: %s <path>\n", addFolderCmdText)
			return
		}
		full, err := expandFolder(cmds[1])
		if err != nil {
			fmt.Println(err)
			return
		}
		state.client.AddFolder(full)
	case setMountCmdText:
		if len(cmds) < 2 {
			fmt.Printf("Usage: %s <path>\n", setMountCmdText)
			return
		}
		full, err := expandFolder(cmds[1])
		if err != nil {
			fmt.Println(err)
			return
		}
		state.client.SetMount(full)
	case "q":
		fmt.Println("Exiting...")
		os.Exit(0)
	}
}

func (state *cmdState) executeEntryType(selected *prompt.Suggest, defaultMode mode.ModeDef) {
	searchResult, valid := selected.Metadata.(*management_v1.SearchResult)
	if valid {
		lookupResult := state.client.Lookup(searchResult.EntryType, searchResult.CorrelationIds)
		switch searchResult.EntryType {
		case management_v1.EntryType_ARTIST:
			state.mode.Set(mode.AlbumMode)
			state.lookupResult = lookupResult.Entries
		case management_v1.EntryType_ALBUM:
			state.mode.Set(mode.SongMode)
			state.lookupResult = lookupResult.Entries
		case management_v1.EntryType_SONG:
			state.mode.Reset()
			state.currentQueue = append(state.currentQueue, lookupResult.Entries...)
		}
	} else {
		state.mode.Set(defaultMode)
		path := selected.Metadata.(string)
		state.currentQueue = append(state.currentQueue, &management_v1.LookupEntry{Path: path})
	}
}

func expandPath(song string) (string, fs.FileInfo, error) {
	dir, base, err := internal.CleanFilePath(song)
	if err != nil {
		return "", nil, err
	}
	full := path.Join(dir, base)
	stat, err := os.Stat(full)

	return full, stat, err
}

func expandFile(song string) (string, error) {
	if strings.HasPrefix(song, "http://") || strings.HasPrefix(song, "https://") {
		return song, nil
	}
	full, stat, err := expandPath(song)

	if stat != nil && stat.Mode().IsDir() {
		return "", fmt.Errorf("cannot add a directory")
	}
	return full, err
}

func expandFolder(song string) (string, error) {
	full, stat, err := expandPath(song)

	if stat != nil && !stat.Mode().IsDir() {
		return "", fmt.Errorf("cannot add a file")
	}
	return full, err
}

func (state *cmdState) checkSpecialOptions(selected *prompt.Suggest) bool {
	switch selected.Text {
	case selectAll:
		results, ok := selected.Metadata.([]*management_v1.LookupEntry)
		if ok {
			state.currentQueue = append(state.currentQueue, results...)
			state.mode.Reset()
			return true
		}
	case back:
		if selected.Metadata == nil {
			state.mode.Reset()
			return true
		}
	default:
		return false
	}

	return false
}

func getPathInput(in string, selected *prompt.Suggest) (string, error) {
	selectedIsSearchResult := false
	if selected != nil {
		// If the metadata is a string, this is a file path completion
		_, isStr := selected.Metadata.(string)
		selectedIsSearchResult = !isStr
	}

	if selectedIsSearchResult {
		return "", nil
	}

	full, err := expandFile(in)
	return full, err
}
