package internal

import (
	"os"
	"path/filepath"
	"strings"

	platune "github.com/aschey/platune/client"
	tea "github.com/charmbracelet/bubbletea"
)

var noResultsStr string = "No results"

type Search struct {
	client *ManagementClient
}

func NewSearch(client *ManagementClient) *Search {
	return &Search{client: client}
}

func (s *Search) ProcessSearchResults(
	args []string,
	selected *platune.SearchResult,
	filesystemCallback func(file string),
	dbCallback func(entries []*platune.LookupEntry),
) (tea.Model, error) {
	allArgs := strings.Join(args, " ")
	_, err := os.Stat(allArgs)
	if err == nil {
		full, err := filepath.Abs(allArgs)
		if err != nil {
			return nil, err
		}
		filesystemCallback(full)
	} else if strings.HasPrefix(allArgs, "http://") || strings.HasPrefix(allArgs, "https://") {
		filesystemCallback(allArgs)
		return NewInfoModel("Added " + allArgs + " to the queue"), nil
	} else {
		if selected == nil {
			results, err := s.client.Search(&platune.SearchRequest{Query: allArgs})
			if err != nil {
				return nil, err
			}
			if len(results.Results) == 0 {
				return NewInfoModel(noResultsStr), nil
			} else if len(results.Results) == 1 {
				selected = results.Results[0]
			} else {
				return s.renderSearchResults(results, dbCallback), nil
			}
		}

		if selected != nil {
			if selected.EntryType == platune.EntryType_SONG {
				lookupResults, _ := s.client.Lookup(selected.EntryType, selected.CorrelationIds)

				dbCallback(lookupResults.Entries)
				return NewInfoModel("Added " + selected.Entry + " " + selected.Description + " to the queue"), nil
			} else if selected.EntryType == platune.EntryType_ARTIST {
				albumArtistResponse, err := s.client.GetAlbumArtistsByNames([]string{selected.Entry})
				if err != nil {
					return nil, err
				}
				if len(albumArtistResponse.Entities) == 0 {
					return NewInfoModel("No albums to show"), nil
				}
				albumArtist := albumArtistResponse.Entities[0]
				albumsResponse, err := s.client.GetAlbumsByAlbumArtists([]int64{albumArtist.Id})
				if err != nil {
					return nil, err
				}
				items := []displayItem{}
				for _, album := range albumsResponse.Entries {
					items = append(items, displayItem{title: album.Album})
				}
				return s.renderDisplay("Albums by "+selected.Entry, items, func(di []displayItem) {}), nil
			}
		}

	}
	return nil, nil
}
