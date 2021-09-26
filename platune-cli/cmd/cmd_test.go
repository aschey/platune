package cmd

import (
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/aschey/go-prompt"
	"github.com/aschey/platune/cli/v2/internal"
	"github.com/aschey/platune/cli/v2/test"
	platune "github.com/aschey/platune/client"
	"github.com/golang/mock/gomock"
)

type completionCase struct {
	in          string
	outLength   int
	choiceIndex int
	choiceText  string
}

var originalArgs = os.Args

func runPlayerTest(t *testing.T, expected string,
	expectFunc func(expect *test.MockPlayerClientMockRecorder), args ...string) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	mock := test.NewMockPlayerClient(ctrl)
	expectFunc(mock.EXPECT())
	internal.Client = internal.NewTestClient(mock, nil)

	runTest(t, expected, args...)
}

func runManagementTest(t *testing.T, expected string,
	expectFunc func(expect *test.MockManagementClientMockRecorder), args ...string) string {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	mock := test.NewMockManagementClient(ctrl)
	expectFunc(mock.EXPECT())
	internal.Client = internal.NewTestClient(nil, mock)

	return runTest(t, expected, args...)
}

func runTest(t *testing.T, expected string, args ...string) string {
	os.Args = append(originalArgs, args...)
	rescueStdout := os.Stdout
	rOut, wOut, _ := os.Pipe()
	rootCmd.SetOut(wOut)
	os.Stdout = wOut

	if err := rootCmd.Execute(); err != nil {
		t.Errorf(err.Error())
	}
	wOut.Close()
	rootCmd.SetOut(rescueStdout)
	os.Stdout = rescueStdout
	var out, _ = io.ReadAll(rOut)
	outStr := string(out)
	if expected != "" && outStr != expected {
		t.Errorf("Expected %s, Got %s", expected, outStr)
	}

	return outStr
}

func executeInteractive(t *testing.T, steps []completionCase) {
	for _, step := range steps {
		buf := prompt.NewBuffer()
		buf.InsertText(step.in, false, true)
		doc := buf.Document()
		results := state.completer(*doc)
		if len(results) != step.outLength {
			t.Errorf("Expected length %d, Got %d", step.outLength, len(results))
		}
		choice := results[step.choiceIndex]
		if choice.Text != step.choiceText {
			t.Errorf("Expected %s, Got %s", step.choiceText, choice.Text)
		}
		state.executor(step.in, &results[step.choiceIndex])
	}
}

func testInteractive(t *testing.T, searchQuery string, searchResults []*platune.SearchResult,
	lookupRequest *platune.LookupRequest, lookupEntries []*platune.LookupEntry,
	matcherFunc func(arg *platune.AddToQueueRequest) bool, steps []completionCase) {
	searchClient = nil

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	mgmtMock := test.NewMockManagementClient(ctrl)
	playerMock := test.NewMockPlayerClient(ctrl)
	stream := test.NewMockManagement_SearchClient(ctrl)
	stream.EXPECT().Send(&platune.SearchRequest{Query: searchQuery}).Return(nil)

	stream.EXPECT().Recv().Return(&platune.SearchResponse{Results: searchResults}, nil)

	mgmtMock.EXPECT().Search(gomock.Any()).Return(stream, nil)
	mgmtMock.EXPECT().
		Lookup(gomock.Any(), lookupRequest).
		Return(&platune.LookupResponse{Entries: lookupEntries}, nil)
	matcher := test.NewMatcher(func(arg interface{}) bool {
		return matcherFunc(arg.(*platune.AddToQueueRequest))
	})
	playerMock.EXPECT().AddToQueue(gomock.Any(), matcher)

	internal.Client = internal.NewTestClient(playerMock, mgmtMock)
	initState()

	executeInteractive(t, steps)
}

func TestAddQueue(t *testing.T) {
	testSong := "root.go"
	runPlayerTest(t, "Added\n", func(expect *test.MockPlayerClientMockRecorder) {
		matcher := test.NewMatcher(func(arg interface{}) bool {
			path, _ := filepath.Abs(testSong)
			return arg.(*platune.AddToQueueRequest).Songs[0] == path
		})
		expect.AddToQueue(gomock.Any(), matcher)
	}, addQueueCmdText, testSong)
}

func TestSetQueue(t *testing.T) {
	testSong1 := "test1"
	testSong2 := "test2"
	runPlayerTest(t, "Queue Set\n", func(expect *test.MockPlayerClientMockRecorder) {
		matcher := test.NewMatcher(func(arg interface{}) bool {
			queue := arg.(*platune.QueueRequest).Queue
			return queue[0] == testSong1 && queue[1] == testSong2
		})
		expect.SetQueue(gomock.Any(), matcher)
	}, setQueueCmdText, testSong1, testSong2)
}

func TestSeek(t *testing.T) {
	testCases := []struct {
		formatStr string
		expected  uint64
	}{
		{"30", 30000},
		{"2:30", 150000},
		{"3:05:30", 11130000},
	}

	for _, tc := range testCases {
		matcher := test.NewMatcher(func(arg interface{}) bool {
			return arg.(*platune.SeekRequest).Millis == tc.expected
		})
		runPlayerTest(t, fmt.Sprintf("Seeked to %s\n", tc.formatStr), func(expect *test.MockPlayerClientMockRecorder) {
			expect.Seek(gomock.Any(), matcher)
		}, seekCmdText, tc.formatStr)
	}

}

func TestResume(t *testing.T) {
	runPlayerTest(t, "Resumed\n", func(expect *test.MockPlayerClientMockRecorder) {
		expect.Resume(gomock.Any(), gomock.Any())
	}, resumeCmdText)
}

func TestPause(t *testing.T) {
	runPlayerTest(t, "Paused\n", func(expect *test.MockPlayerClientMockRecorder) {
		expect.Pause(gomock.Any(), gomock.Any())
	}, pauseCmdText)
}

func TestNext(t *testing.T) {
	runPlayerTest(t, "Next\n", func(expect *test.MockPlayerClientMockRecorder) {
		expect.Next(gomock.Any(), gomock.Any())
	}, nextCmdText)
}

func TestPrevious(t *testing.T) {
	runPlayerTest(t, "Previous\n", func(expect *test.MockPlayerClientMockRecorder) {
		expect.Previous(gomock.Any(), gomock.Any())
	}, previousCmdText)
}

func TestStop(t *testing.T) {
	runPlayerTest(t, "Stopped\n", func(expect *test.MockPlayerClientMockRecorder) {
		expect.Stop(gomock.Any(), gomock.Any())
	}, stopCmdText)
}

func TestSync(t *testing.T) {
	res := runManagementTest(t, "", func(expect *test.MockManagementClientMockRecorder) {
		ctrl := gomock.NewController(t)
		stream := test.NewMockManagement_SyncClient(ctrl)
		stream.EXPECT().Recv().Return(&platune.Progress{Percentage: 0.1}, nil)
		stream.EXPECT().Recv().Return(nil, fmt.Errorf("error"))
		expect.Sync(gomock.Any(), gomock.Any()).Return(stream, nil)
	}, syncCmdText)
	if len(res) == 0 {
		t.Errorf("Expected length > 0")
	}
}

func TestGetAllFolders(t *testing.T) {
	response := "C://test"
	res := runManagementTest(t, "", func(expect *test.MockManagementClientMockRecorder) {
		expect.GetAllFolders(gomock.Any(), gomock.Any()).Return(&platune.FoldersMessage{Folders: []string{response}}, nil)
	}, getAllFoldersCmdText)
	if !strings.Contains(res, response) {
		t.Errorf("Response should contain folder")
	}
}

func TestAddFolder(t *testing.T) {
	folder := "folder1"
	runManagementTest(t, "Added\n", func(expect *test.MockManagementClientMockRecorder) {
		matcher := test.NewMatcher(func(arg interface{}) bool {
			folders := arg.(*platune.FoldersMessage).Folders
			return folders[0] == folder
		})
		expect.AddFolders(gomock.Any(), matcher)
	}, addFolderCmdText, folder)
}

func TestSetMount(t *testing.T) {
	folder := "/home/test"
	runManagementTest(t, "Set\n", func(expect *test.MockManagementClientMockRecorder) {
		matcher := test.NewMatcher(func(arg interface{}) bool {
			mount := arg.(*platune.RegisteredMountMessage).Mount
			return mount == folder
		})
		expect.RegisterMount(gomock.Any(), matcher)
	}, setMountCmdText, folder)
}

func TestAddQueueFileCompleter(t *testing.T) {
	searchClient = nil
	buf := prompt.NewBuffer()
	buf.InsertText(addQueueCmdText+" root", false, true)
	doc := buf.Document()

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	mock := test.NewMockManagementClient(ctrl)
	stream := test.NewMockManagement_SearchClient(ctrl)
	stream.EXPECT().Send(gomock.Any()).Return(nil)
	stream.EXPECT().Recv().Return(&platune.SearchResponse{Results: []*platune.SearchResult{}}, nil)

	mock.EXPECT().Search(gomock.Any()).Return(stream, nil)
	internal.Client = internal.NewTestClient(nil, mock)
	initState()
	results := state.completer(*doc)
	if len(results) != 1 {
		t.Error("Should've found one result")
	}
	if results[0].Text != "root.go" {
		t.Error("Result should be root.go")
	}
}

func TestAddQueueSongSelection(t *testing.T) {
	artist := "blah"
	searchResults := []*platune.SearchResult{
		{Entry: "song name", EntryType: platune.EntryType_SONG, Artist: &artist, CorrelationIds: []int32{1}, Description: "song desc"},
	}
	lookupRequest := &platune.LookupRequest{EntryType: platune.EntryType_SONG, CorrelationIds: []int32{1}}
	lookupEntries := []*platune.LookupEntry{
		{Artist: "artist name", Album: "album 1", Song: "song name", Path: "/test/path/1", Track: 1},
	}
	matcherFunc := func(arg *platune.AddToQueueRequest) bool {
		return arg.Songs[0] == "/test/path/1"
	}
	steps := []completionCase{
		{in: addQueueCmdText + " song name", outLength: 1, choiceText: "song name", choiceIndex: 0},
	}
	testInteractive(t, "song name", searchResults, lookupRequest, lookupEntries, matcherFunc, steps)
}

func TestAddQueueAlbumSelection(t *testing.T) {
	artist := "blah"
	searchResults := []*platune.SearchResult{
		{Entry: "album name", EntryType: platune.EntryType_ALBUM, Artist: &artist, CorrelationIds: []int32{1}, Description: "album desc"},
	}
	lookupRequest := &platune.LookupRequest{EntryType: platune.EntryType_ALBUM, CorrelationIds: []int32{1}}
	lookupEntries := []*platune.LookupEntry{
		{Artist: "artist name", Album: "album name", Song: "track 1", Path: "/test/path/1", Track: 1},
		{Artist: "artist name", Album: "album name", Song: "track 2", Path: "/test/path/2", Track: 2},
	}
	matcherFunc := func(arg *platune.AddToQueueRequest) bool {
		return arg.Songs[0] == "/test/path/1"
	}
	steps := []completionCase{
		{in: addQueueCmdText + " album name", outLength: 1, choiceText: "album name", choiceIndex: 0},
		{in: "track 1", outLength: 1, choiceText: "track 1", choiceIndex: 0},
	}
	testInteractive(t, "album name", searchResults, lookupRequest, lookupEntries, matcherFunc, steps)
}

func TestAddQueueArtistSelection(t *testing.T) {
	searchResults := []*platune.SearchResult{
		{Entry: "artist name", EntryType: platune.EntryType_ARTIST, CorrelationIds: []int32{1}, Description: "artist desc"},
	}
	lookupRequest := &platune.LookupRequest{EntryType: platune.EntryType_ARTIST, CorrelationIds: []int32{1}}
	lookupEntries := []*platune.LookupEntry{
		{Artist: "artist name", Album: "album 1", Song: "track 1", Path: "/test/path/1", Track: 1},
		{Artist: "artist name", Album: "album 2", Song: "track 1", Path: "/test/path/2", Track: 1},
	}
	matcherFunc := func(arg *platune.AddToQueueRequest) bool {
		return arg.Songs[0] == "/test/path/1"
	}
	steps := []completionCase{
		{in: addQueueCmdText + " artist name", outLength: 1, choiceText: "artist name", choiceIndex: 0},
		{in: "album 1", outLength: 1, choiceText: "album 1", choiceIndex: 0},
		{in: "track 1", outLength: 1, choiceText: "track 1", choiceIndex: 0},
	}
	testInteractive(t, "artist name", searchResults, lookupRequest, lookupEntries, matcherFunc, steps)
}

func TestAddFolderCompleter(t *testing.T) {
	initState()
	buf := prompt.NewBuffer()
	buf.InsertText(addFolderCmdText+" root", false, true)
	doc := buf.Document()

	results := state.completer(*doc)
	if len(results) != 1 {
		t.Error("Should've found one result")
	}
	if results[0].Text != "root.go" {
		t.Error("Result should be root.go")
	}
}

func TestSetQueueCompleter(t *testing.T) {
	initState()
	state.mode = SetQueueMode

	buf := prompt.NewBuffer()
	buf.InsertText("root", false, true)
	doc := buf.Document()

	results := state.completer(*doc)
	if len(results) != 1 {
		t.Error("Should've found one result")
	}
}

func TestSetQueueExecutor(t *testing.T) {
	initState()
	state.executor(setQueueCmdText, nil)
	if state.mode != setQueueCmdText+"> " {
		t.Error(fmt.Sprintf("Live prefix should be set to %s> ", setQueueCmdText))
	}
	state.executor("root.go", nil)
	if len(state.currentQueue) != 1 {
		t.Error("Should've added an item to the queue")
	}
	if !strings.HasSuffix(state.currentQueue[0], "root.go") {
		t.Error("root.go should've been added to the queue")
	}
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	mock := test.NewMockPlayerClient(ctrl)
	matcher := test.NewMatcher(func(arg interface{}) bool {
		queue := arg.(*platune.QueueRequest).Queue
		return strings.HasSuffix(queue[0], "root.go")
	})
	mock.EXPECT().SetQueue(gomock.Any(), matcher)
	internal.Client = internal.NewTestClient(mock, nil)
	state.executor("", nil)
}
