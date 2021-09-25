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
	if state.mode != NormalMode {
		state.executeMode(in, selected)
		return
	}

	cmds := strings.SplitN(in, " ", 2)
	if len(cmds) == 0 {
		return
	}

	state.executeCmd(cmds, selected)
}

func (state *cmdState) executeMode(in string, selected *prompt.Suggest) {
	switch state.mode {
	case SetQueueMode:
		if strings.Trim(in, " ") == "" {
			internal.Client.SetQueue(state.currentQueue)
			state.currentQueue = []string{}
			state.mode = NormalMode
		} else {
			in, err := expandFile(in)
			if err != nil {
				fmt.Println(err)
				return
			}

			state.currentQueue = append(state.currentQueue, in)
			fmt.Println(internal.PrettyPrintList(state.currentQueue))
		}

	case AlbumMode:
		if selected.Text == "(Select All)" {
			results := selected.Metadata.([]*platune.LookupEntry)
			internal.Client.AddSearchResultsToQueue(results)
			state.mode = NormalMode
			return
		}
		state.mode = SongMode
		newResults := []*platune.LookupEntry{}
		for _, r := range state.lookupResult {
			if r.Album == in {
				newResults = append(newResults, r)
			}
		}
		state.lookupResult = newResults
		return
	case SongMode:
		if selected.Text == "(Select All)" {
			results := selected.Metadata.([]*platune.LookupEntry)
			internal.Client.AddSearchResultsToQueue(results)
			state.mode = NormalMode
			return
		}
		state.mode = NormalMode
		lookupResponse := selected.Metadata.(*platune.LookupEntry)
		internal.Client.AddToQueue([]string{lookupResponse.Path})
		return
	}
}

func (state *cmdState) executeCmd(cmds []string, selected *prompt.Suggest) {
	switch cmds[0] {
	case "set-queue":
		fmt.Println("Enter file paths or urls to add to the queue.")
		fmt.Println("Enter a blank line when done.")
		state.mode = SetQueueMode
		return
	case "add-queue":
		if len(cmds) < 2 {
			fmt.Println("Usage: add-queue <path or url>")
			return
		}

		if selected != nil {
			searchResult := selected.Metadata.(*platune.SearchResult)
			lookupResult := internal.Client.Lookup(searchResult.EntryType, searchResult.CorrelationIds)
			switch searchResult.EntryType {
			case platune.EntryType_ARTIST, platune.EntryType_ALBUM_ARTIST:
				state.mode = AlbumMode
				state.lookupResult = lookupResult.Entries
			case platune.EntryType_ALBUM:
				state.mode = SongMode
				state.lookupResult = lookupResult.Entries
			case platune.EntryType_SONG:
				state.mode = NormalMode
				internal.Client.AddSearchResultsToQueue(lookupResult.Entries)
			}

			return
		}

		full, err := expandFile(cmds[1])
		if err != nil {
			fmt.Println(err)
			return
		}
		internal.Client.AddToQueue([]string{full})
	case "seek":
		if len(cmds) < 2 {
			fmt.Println("Usage: seek [hh]:[mm]:ss")
			return
		}
		internal.Client.Seek(cmds[1])
	case "pause":
		internal.Client.Pause()
	case "resume":
		internal.Client.Resume()
	case "stop":
		internal.Client.Stop()
	case "next":
		internal.Client.Next()
	case "previous":
		internal.Client.Previous()
	case "sync":
		SyncProgress()
		fmt.Println()
	case "get-all-folders":
		internal.Client.GetAllFolders()
	case "add-folder":
		if len(cmds) < 2 {
			fmt.Println("Usage: add-folder <path>")
			return
		}
		full, err := expandFolder(cmds[1])
		if err != nil {
			fmt.Println(err)
			return
		}
		internal.Client.AddFolder(full)
	case "set-mount":
		if len(cmds) < 2 {
			fmt.Println("Usage: set-mount <path>")
			return
		}
		full, err := expandFolder(cmds[1])
		if err != nil {
			fmt.Println(err)
			return
		}
		internal.Client.SetMount(full)
	case "q":
		fmt.Println("Exiting...")
		os.Exit(0)
	}
}

func expandPath(song string) (string, fs.FileInfo, error) {
	if strings.HasPrefix(song, "http") {
		return song, nil, nil
	}

	dir, base, err := internal.CleanFilePath(song)

	if err != nil {
		return "", nil, err
	}
	full := path.Join(dir, base)
	stat, err := os.Stat(full)

	return full, stat, err
}

func expandFile(song string) (string, error) {
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
