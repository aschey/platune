package search

import (
	"bytes"
	"fmt"
	"io"
	"path/filepath"
	"testing"

	"github.com/MarvinJWendt/testza"
	"github.com/aschey/platune/cli/v2/internal"
	"github.com/aschey/platune/cli/v2/test"
	platune "github.com/aschey/platune/client"
	"github.com/charmbracelet/bubbles/list"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/golang/mock/gomock"
)

func testRenderItem(t *testing.T, index int, expected string) {
	results := []*platune.SearchResult{
		{
			Entry:          "test entry1",
			Description:    "test description1",
			EntryType:      platune.EntryType_SONG,
			CorrelationIds: []int32{1},
		},
		{
			Entry:          "test entry2",
			Description:    "test description2",
			EntryType:      platune.EntryType_SONG,
			CorrelationIds: []int32{1},
		},
	}
	items := getItems(results)

	d := itemDelegate{}
	l := list.NewModel(items, d, 0, 0)

	var buf bytes.Buffer
	d.Render(&buf, l, index, items[index])

	out := buf.String()
	testza.AssertEqual(t, expected, out, fmt.Sprintf("Expected %s, got %s", expected, out))
}

func TestRenderSelected(t *testing.T) {
	expected := selectedItemStyle.Render("▶ test entry1 - test description1")
	testRenderItem(t, 0, expected)
}

func TestRender(t *testing.T) {
	expected := itemStyle.Render("test entry2 - test description2")
	testRenderItem(t, 1, expected)
}

func TestSelectOneItem(t *testing.T) {
	results := []*platune.SearchResult{
		{
			Entry:          "test entry1",
			Description:    "test description1",
			EntryType:      platune.EntryType_SONG,
			CorrelationIds: []int32{1},
		},
		{
			Entry:          "test entry2",
			Description:    "test description2",
			EntryType:      platune.EntryType_SONG,
			CorrelationIds: []int32{1},
		},
	}
	items := getItems(results)

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	mock := test.NewMockManagementClient(ctrl)
	lookupRequest := &platune.LookupRequest{
		EntryType:      platune.EntryType_SONG,
		CorrelationIds: []int32{1},
	}
	lookupEntries := []*platune.LookupEntry{
		{
			Artist: "artist name",
			Album:  "album 1",
			Song:   "song name",
			Path:   "/test/path/1",
			Track:  1,
		},
	}
	mock.EXPECT().
		Lookup(gomock.Any(), lookupRequest).
		Return(&platune.LookupResponse{Entries: lookupEntries}, nil)
	client := internal.NewTestClient(nil, mock)

	d := itemDelegate{}
	l := list.NewModel(items, d, 0, 0)
	m := model{list: l, client: &client, callback: func(entries []*platune.LookupEntry) {}}

	m.list.CursorDown()

	m.Update(tea.KeyMsg{Type: tea.KeyEnter})
}

func TestProcessFilesystem(t *testing.T) {
	selectedFile := ""
	fsCallback := func(file string) { selectedFile = file }
	fileToChoose := "./search.go"
	search := NewSearch(nil)
	search.ProcessSearchResults([]string{fileToChoose}, fsCallback, nil)
	fullPath, _ := filepath.Abs(selectedFile)

	testza.AssertEqual(t, fullPath, selectedFile,
		fmt.Sprintf("Expected %s got %s", fullPath, selectedFile))
}

func TestOneSearchResult(t *testing.T) {
	lookupEntries := []*platune.LookupEntry{}
	dbCallback := func(entries []*platune.LookupEntry) { lookupEntries = entries }

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	stream := test.NewMockManagement_SearchClient(ctrl)
	song := "test song"
	stream.EXPECT().Send(&platune.SearchRequest{Query: song}).Return(nil)
	searchResult := &platune.SearchResult{
		Entry:          song,
		EntryType:      platune.EntryType_SONG,
		Description:    "test description",
		CorrelationIds: []int32{1},
	}
	stream.EXPECT().
		Recv().
		Return(&platune.SearchResponse{Results: []*platune.SearchResult{searchResult}}, nil)
	mock := test.NewMockManagementClient(ctrl)
	mock.EXPECT().Search(gomock.Any()).Return(stream, nil)
	mock.EXPECT().
		Lookup(gomock.Any(), &platune.LookupRequest{EntryType: platune.EntryType_SONG, CorrelationIds: []int32{1}}).
		Return(&platune.LookupResponse{Entries: []*platune.LookupEntry{{Song: song}}}, nil)

	client := internal.NewTestClient(nil, mock)
	search := NewSearch(&client)

	search.ProcessSearchResults([]string{song}, nil, dbCallback)

	testza.AssertLen(t, lookupEntries, 1)
	testza.AssertEqual(t, song, lookupEntries[0].Song)
}

func TestNoResults(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	stream := test.NewMockManagement_SearchClient(ctrl)
	stream.EXPECT().Send(&platune.SearchRequest{Query: "test song"}).Return(nil)

	stream.EXPECT().Recv().Return(&platune.SearchResponse{Results: []*platune.SearchResult{}}, nil)
	mock := test.NewMockManagementClient(ctrl)
	mock.EXPECT().Search(gomock.Any()).Return(stream, nil)

	client := internal.NewTestClient(nil, mock)
	search := NewSearch(&client)

	outStr, _ := testza.CaptureStdout(func(io.Writer) error {
		search.ProcessSearchResults([]string{"test song"}, nil, nil)
		return nil
	})

	testza.AssertEqual(
		t,
		noResultsStr+"\n",
		outStr,
		fmt.Sprintf("Expected %s, got %s", noResultsStr, outStr),
	)
}
