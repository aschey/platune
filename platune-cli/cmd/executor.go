package cmd

import (
	"fmt"
	"io/fs"
	"os"
	"path"
	"strings"

	"github.com/aschey/go-prompt"
	"github.com/aschey/platune/cli/v2/internal"
	platune "github.com/aschey/platune/client"
)

func (state *cmdState) executor(in string, selected *prompt.Suggest) {
	isSetQueueMode := state.mode[0] == SetQueueMode
	if selected == nil {
		// User did not explicitly choose a result
		// See if we can find a match instead
		text := strings.Trim(strings.ToLower(in), " ")
		if strings.HasPrefix(text, addQueueCmdText) {
			cmds := strings.SplitN(text, " ", 2)
			text = strings.Trim(cmds[len(cmds)-1], " ")
		}
		for _, suggestion := range state.suggestions {
			if strings.ToLower(suggestion.Text) == text {
				selected = &suggestion
				break
			}
		}
	}
	if state.mode[len(state.mode)-1] != NormalMode {
		state.executeMode(in, selected)
	} else {
		cmds := strings.SplitN(in, " ", 2)
		if len(cmds) == 0 {
			return
		}

		state.executeCmd(cmds, selected)
	}

	if state.mode[0] == NormalMode && len(state.currentQueue) > 0 {
		if isSetQueueMode {
			state.client.SetQueueFromSearchResults(state.currentQueue, true)
		} else {
			state.client.AddSearchResultsToQueue(state.currentQueue, true)
		}

		state.currentQueue = []*platune.LookupEntry{}
	}
}

func (state *cmdState) executeMode(in string, selected *prompt.Suggest) {
	switch state.mode[len(state.mode)-1] {
	case SetQueueMode:
		if strings.Trim(in, " ") == "" {
			state.mode = []Mode{NormalMode}
		} else if selected != nil {
			state.executeEntryType(selected, SetQueueMode)
		} else {
			state.currentQueue = append(state.currentQueue, &platune.LookupEntry{Path: in})
		}

	case AlbumMode:
		if state.checkSpecialOptions(selected) {
			return
		}
		state.mode = append(state.mode, SongMode)
		newResults := []*platune.LookupEntry{}
		for _, r := range state.lookupResult {
			album := r.Album
			if strings.Trim(r.Album, " ") == "" {
				album = "[Untitled]"
			}
			if album == in {
				newResults = append(newResults, r)
			}
		}
		state.lookupResult = newResults
		return
	case SongMode:
		if state.checkSpecialOptions(selected) {
			return
		}
		newMode := state.mode[0]
		lookupResponse := selected.Metadata.(*platune.LookupEntry)
		state.mode = []Mode{newMode}
		state.currentQueue = append(state.currentQueue, lookupResponse)

		return
	}
}

func (state *cmdState) executeCmd(cmds []string, selected *prompt.Suggest) {
	switch cmds[0] {
	case setQueueCmdText:
		fmt.Println("Enter file paths or urls to add to the queue.")
		fmt.Println("Enter a blank line when done.")
		state.mode = []Mode{SetQueueMode}
		return
	case addQueueCmdText:
		if len(cmds) < 2 {
			fmt.Printf("Usage: %s <path or url>\n", addQueueCmdText)
			return
		}

		if selected != nil {
			state.executeEntryType(selected, NormalMode)
			return
		}

		full, err := expandFile(cmds[1])
		if err != nil {
			fmt.Println(err)
			return
		}
		state.client.AddToQueue([]string{full}, true)
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

func (state *cmdState) executeEntryType(selected *prompt.Suggest, defaultMode Mode) {
	searchResult, valid := selected.Metadata.(*platune.SearchResult)
	if valid {
		lookupResult := state.client.Lookup(searchResult.EntryType, searchResult.CorrelationIds)
		switch searchResult.EntryType {
		case platune.EntryType_ARTIST, platune.EntryType_ALBUM_ARTIST:
			state.mode = append(state.mode, AlbumMode)
			state.lookupResult = lookupResult.Entries
		case platune.EntryType_ALBUM:
			state.mode = append(state.mode, SongMode)
			state.lookupResult = lookupResult.Entries
		case platune.EntryType_SONG:
			state.mode = []Mode{defaultMode}
			state.currentQueue = append(state.currentQueue, lookupResult.Entries...)
		}
	} else {
		path := selected.Metadata.(string)
		state.currentQueue = append(state.currentQueue, &platune.LookupEntry{Path: path})
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
		results, ok := selected.Metadata.([]*platune.LookupEntry)
		if ok {
			state.currentQueue = append(state.currentQueue, results...)
			state.mode = []Mode{state.mode[0]}
			return true
		}
	case back:
		if selected.Metadata == nil {
			state.mode = []Mode{state.mode[0]}
			return true
		}
	default:
		return false
	}

	return false
}
