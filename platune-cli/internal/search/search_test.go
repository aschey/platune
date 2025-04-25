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
	management_v1 "github.com/aschey/platune/client/management_v1"
	"github.com/charmbracelet/bubbles/list"
	tea "github.com/charmbracelet/bubbletea"
	"go.uber.org/mock/gomock"
)

func testRenderItem(t *testing.T, index int, expected string) {
	results := []*management_v1.SearchResult{
		{
			Entry:          "test entry1",
			Description:    "test description1",
			EntryType:      management_v1.EntryType_SONG,
			CorrelationIds: []int64{1},
		},
		{
			Entry:          "test entry2",
			Description:    "test description2",
			EntryType:      management_v1.EntryType_SONG,
			CorrelationIds: []int64{1},
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
	results := []*management_v1.SearchResult{
		{
			Entry:          "test entry1",
			Description:    "test description1",
			EntryType:      management_v1.EntryType_SONG,
			CorrelationIds: []int64{1},
		},
		{
			Entry:          "test entry2",
			Description:    "test description2",
			EntryType:      management_v1.EntryType_SONG,
			CorrelationIds: []int64{1},
		},
	}
	items := getItems(results)

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	mock := test.NewMockManagementClient(ctrl)
	lookupRequest := &management_v1.LookupRequest{
		EntryType:      management_v1.EntryType_SONG,
		CorrelationIds: []int64{1},
	}
	lookupEntries := []*management_v1.LookupEntry{
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
		Return(&management_v1.LookupResponse{Entries: lookupEntries}, nil)
	client := internal.NewTestClient(nil, mock)

	d := itemDelegate{}
	l := list.NewModel(items, d, 0, 0)
	m := model{list: l, client: &client, callback: func(entries []*management_v1.LookupEntry) {}}

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
	lookupEntries := []*management_v1.LookupEntry{}
	dbCallback := func(entries []*management_v1.LookupEntry) { lookupEntries = entries }

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	stream := test.NewMockBidiStreamingClient[management_v1.SearchRequest, management_v1.SearchResponse](ctrl)
	song := "test song"
	stream.EXPECT().Send(&management_v1.SearchRequest{Query: song}).Return(nil)
	searchResult := &management_v1.SearchResult{
		Entry:          song,
		EntryType:      management_v1.EntryType_SONG,
		Description:    "test description",
		CorrelationIds: []int64{1},
	}
	stream.EXPECT().
		Recv().
		Return(&management_v1.SearchResponse{Results: []*management_v1.SearchResult{searchResult}}, nil)
	mock := test.NewMockManagementClient(ctrl)
	mock.EXPECT().Search(gomock.Any()).Return(stream, nil)
	mock.EXPECT().
		Lookup(gomock.Any(), &management_v1.LookupRequest{EntryType: management_v1.EntryType_SONG, CorrelationIds: []int64{1}}).
		Return(&management_v1.LookupResponse{Entries: []*management_v1.LookupEntry{{Song: song}}}, nil)

	client := internal.NewTestClient(nil, mock)
	search := NewSearch(&client)

	search.ProcessSearchResults([]string{song}, nil, dbCallback)

	testza.AssertLen(t, lookupEntries, 1)
	testza.AssertEqual(t, song, lookupEntries[0].Song)
}

func TestNoResults(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	stream := test.NewMockBidiStreamingClient[management_v1.SearchRequest, management_v1.SearchResponse](ctrl)
	stream.EXPECT().Send(&management_v1.SearchRequest{Query: "test song"}).Return(nil)

	stream.EXPECT().Recv().Return(&management_v1.SearchResponse{Results: []*management_v1.SearchResult{}}, nil)
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
