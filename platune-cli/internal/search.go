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
				return s.handleSearchResult(selected, dbCallback)
			} else {
				return s.renderSearchResults(results, dbCallback), nil
			}
		} else {
			return s.handleSearchResult(selected, dbCallback)
		}
	}
	return nil, nil
}

func (s *Search) handleSearchResult(searchResult *platune.SearchResult, dbCallback func(entries []*platune.LookupEntry)) (tea.Model, error) {
	switch searchResult.EntryType {
	case platune.EntryType_SONG:
		lookupResults, _ := s.client.Lookup(searchResult.EntryType, searchResult.CorrelationIds)

		dbCallback(lookupResults.Entries)
		return NewInfoModel("Added " + searchResult.Entry + " " + searchResult.Description + " to the queue"), nil
	case platune.EntryType_ARTIST:
		albumArtistIds := searchResult.CorrelationIds
		albumsResponse, err := s.client.GetAlbumsByAlbumArtists(albumArtistIds)
		if err != nil {
			return nil, err
		}
		items := []displayItem{}
		for _, album := range albumsResponse.Entries {
			items = append(items, displayItem{title: album.Album})
		}
		return s.renderDisplay("Albums by "+searchResult.Entry, items, func(di []displayItem) {}), nil
	case platune.EntryType_ALBUM:
		lookupResults, _ := s.client.Lookup(searchResult.EntryType, searchResult.CorrelationIds)
		items := []displayItem{}
		for _, song := range lookupResults.Entries {
			items = append(items, displayItem{title: song.Song})
		}
		return s.renderDisplay(searchResult.Entry, items, func(di []displayItem) {}), nil
	}
	return nil, nil
}
