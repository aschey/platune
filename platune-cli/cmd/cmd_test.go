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

func initSetQueuePrompt(t *testing.T) {
	state.executor(setQueueCmdText, nil)
	if state.mode[0] != setQueueCmdText+"> " {
		t.Error(fmt.Sprintf("Live prefix should be set to %s> ", setQueueCmdText))
	}
}

func executeInteractive(t *testing.T, steps []completionCase, selectPrompt bool) {
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
		if selectPrompt {
			state.executor(step.in, &results[step.choiceIndex])
		} else {
			state.executor(step.in, nil)
		}
	}
}

func testInteractive(t *testing.T, searchQuery string, searchResults []*platune.SearchResult,
	lookupRequest *platune.LookupRequest, lookupEntries []*platune.LookupEntry,
	matcherFunc func(arg interface{}) bool, steps []completionCase, isAddQueue bool, selectPrompt bool) {
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
		return matcherFunc(arg)
	})
	if isAddQueue {
		playerMock.EXPECT().AddToQueue(gomock.Any(), matcher)
	} else {
		playerMock.EXPECT().SetQueue(gomock.Any(), matcher)
	}

	internal.Client = internal.NewTestClient(playerMock, mgmtMock)

	executeInteractive(t, steps, selectPrompt)

	if !isAddQueue {
		state.executor("", nil)
	}
}

func TestAddQueueFile(t *testing.T) {
	testSong := "root.go"
	runPlayerTest(t, "Added\n", func(expect *test.MockPlayerClientMockRecorder) {
		matcher := test.NewMatcher(func(arg interface{}) bool {
			path, _ := filepath.Abs(testSong)
			return arg.(*platune.AddToQueueRequest).Songs[0] == path
		})
		expect.AddToQueue(gomock.Any(), matcher)
	}, addQueueCmdText, testSong)
}

func TestAddQueueUrl(t *testing.T) {
	testSong := "http://test.com/blah.mp3"
	runPlayerTest(t, "Added\n", func(expect *test.MockPlayerClientMockRecorder) {
		matcher := test.NewMatcher(func(arg interface{}) bool {
			return arg.(*platune.AddToQueueRequest).Songs[0] == testSong
		})
		expect.AddToQueue(gomock.Any(), matcher)
	}, addQueueCmdText, testSong)
}

func TestSetQueueFile(t *testing.T) {
	testSong := "root.go"
	runPlayerTest(t, "Queue Set\n", func(expect *test.MockPlayerClientMockRecorder) {
		matcher := test.NewMatcher(func(arg interface{}) bool {
			queue := arg.(*platune.QueueRequest).Queue
			path, _ := filepath.Abs(testSong)
			return queue[0] == path
		})
		expect.SetQueue(gomock.Any(), matcher)
	}, setQueueCmdText, testSong)
}

func TestSetQueueUrl(t *testing.T) {
	testSong := "http://test.com/blah.mp3"
	runPlayerTest(t, "Queue Set\n", func(expect *test.MockPlayerClientMockRecorder) {
		matcher := test.NewMatcher(func(arg interface{}) bool {
			queue := arg.(*platune.QueueRequest).Queue
			return queue[0] == testSong
		})
		expect.SetQueue(gomock.Any(), matcher)
	}, setQueueCmdText, testSong)
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
		expect.GetDeleted(gomock.Any(), gomock.Any()).Return(&platune.GetDeletedResponse{
			Results: []*platune.DeletedResult{},
		}, nil)
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

func TestSetVolume(t *testing.T) {
	volume := 0.5
	runPlayerTest(t, "Set\n", func(expect *test.MockPlayerClientMockRecorder) {
		matcher := test.NewMatcher(func(arg interface{}) bool {
			volumeArg := arg.(*platune.SetVolumeRequest).Volume
			return volumeArg == float32(volume)
		})
		expect.SetVolume(gomock.Any(), matcher)
	}, setVolumeCmdText, fmt.Sprintf("%f", volume))
}

func testFileCompleter(t *testing.T, prefix string, isAddQueue bool) {
	initState()
	searchClient = nil
	buf := prompt.NewBuffer()
	buf.InsertText(prefix+"root", false, true)
	doc := buf.Document()

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	mock := test.NewMockManagementClient(ctrl)
	stream := test.NewMockManagement_SearchClient(ctrl)
	stream.EXPECT().Send(gomock.Any()).Return(nil)
	stream.EXPECT().Recv().Return(&platune.SearchResponse{Results: []*platune.SearchResult{}}, nil)

	mock.EXPECT().Search(gomock.Any()).Return(stream, nil)
	internal.Client = internal.NewTestClient(nil, mock)

	if !isAddQueue {
		initSetQueuePrompt(t)
	}

	results := state.completer(*doc)
	if len(results) != 1 {
		t.Error("Should've found one result")
	}
	if results[0].Text != "root.go" {
		t.Error("Result should be root.go")
	}
}

func TestSetQueueFileCompleter(t *testing.T) {
	testFileCompleter(t, "", false)
}

func TestAddQueueFileCompleter(t *testing.T) {
	testFileCompleter(t, addQueueCmdText+" ", true)
}

func testSongSelection(t *testing.T, matcherFunc func(arg interface{}) bool, prefix string, isAddQueue bool, selectPrompt bool) {
	artist := "blah"
	searchResults := []*platune.SearchResult{
		{Entry: "song name", EntryType: platune.EntryType_SONG, Artist: &artist, CorrelationIds: []int32{1}, Description: "song desc"},
	}
	lookupRequest := &platune.LookupRequest{EntryType: platune.EntryType_SONG, CorrelationIds: []int32{1}}
	lookupEntries := []*platune.LookupEntry{
		{Artist: "artist name", Album: "album 1", Song: "song name", Path: "/test/path/1", Track: 1},
	}

	steps := []completionCase{
		{in: prefix + "song name", outLength: 1, choiceText: "song name", choiceIndex: 0},
	}
	initState()
	if !isAddQueue {
		initSetQueuePrompt(t)
	}
	testInteractive(t, "song name", searchResults, lookupRequest, lookupEntries, matcherFunc, steps, isAddQueue, selectPrompt)
}

func TestAddQueueSongSelection(t *testing.T) {
	matcherFunc := func(arg interface{}) bool {
		return arg.(*platune.AddToQueueRequest).Songs[0] == "/test/path/1"
	}
	testSongSelection(t, matcherFunc, addQueueCmdText+" ", true, true)
	testSongSelection(t, matcherFunc, addQueueCmdText+" ", true, false)
}

func TestSetQueueSongSelection(t *testing.T) {
	matcherFunc := func(arg interface{}) bool {
		return arg.(*platune.QueueRequest).Queue[0] == "/test/path/1"
	}
	testSongSelection(t, matcherFunc, "", false, true)
	testSongSelection(t, matcherFunc, "", false, false)
}

func testAlbumSelection(t *testing.T, matcherFunc func(arg interface{}) bool, prefix string, isAddQueue bool, selectPrompt bool) {
	artist := "blah"
	searchResults := []*platune.SearchResult{
		{Entry: "album name", EntryType: platune.EntryType_ALBUM, Artist: &artist, CorrelationIds: []int32{1}, Description: "album desc"},
	}
	lookupRequest := &platune.LookupRequest{EntryType: platune.EntryType_ALBUM, CorrelationIds: []int32{1}}
	lookupEntries := []*platune.LookupEntry{
		{Artist: "artist name", Album: "album name", Song: "track 1", Path: "/test/path/1", Track: 1},
		{Artist: "artist name", Album: "album name", Song: "track 2", Path: "/test/path/2", Track: 2},
	}

	steps := []completionCase{
		{in: prefix + "album name", outLength: 1, choiceText: "album name", choiceIndex: 0},
		{in: "track 1", outLength: 1, choiceText: "track 1", choiceIndex: 0},
	}
	initState()
	if !isAddQueue {
		initSetQueuePrompt(t)
	}
	testInteractive(t, "album name", searchResults, lookupRequest, lookupEntries, matcherFunc, steps, isAddQueue, selectPrompt)
}

func TestAddQueueAlbumSelection(t *testing.T) {
	matcherFunc := func(arg interface{}) bool {
		req := arg.(*platune.AddToQueueRequest)
		return len(req.Songs) == 1 && req.Songs[0] == "/test/path/1"
	}
	testAlbumSelection(t, matcherFunc, addQueueCmdText+" ", true, true)
	testAlbumSelection(t, matcherFunc, addQueueCmdText+" ", true, false)
}

func TestSetQueueAlbumSelection(t *testing.T) {
	matcherFunc := func(arg interface{}) bool {
		req := arg.(*platune.QueueRequest)
		return len(req.Queue) == 1 && req.Queue[0] == "/test/path/1"
	}
	testAlbumSelection(t, matcherFunc, "", false, true)
	testAlbumSelection(t, matcherFunc, "", false, false)
}

func testAlbumSelectAll(t *testing.T, matcherFunc func(arg interface{}) bool, prefix string, isAddQueue bool, selectPrompt bool) {
	artist := "blah"
	searchResults := []*platune.SearchResult{
		{Entry: "album name", EntryType: platune.EntryType_ALBUM, Artist: &artist, CorrelationIds: []int32{1}, Description: "album desc"},
	}
	lookupRequest := &platune.LookupRequest{EntryType: platune.EntryType_ALBUM, CorrelationIds: []int32{1}}
	lookupEntries := []*platune.LookupEntry{
		{Artist: "artist name", Album: "album name", Song: "track 1", Path: "/test/path/1", Track: 1},
		{Artist: "artist name", Album: "album name", Song: "track 2", Path: "/test/path/2", Track: 2},
	}

	steps := []completionCase{
		{in: prefix + "album name", outLength: 1, choiceText: "album name", choiceIndex: 0},
		{in: selectAll, outLength: 1, choiceText: selectAll, choiceIndex: 0},
	}
	initState()
	if !isAddQueue {
		initSetQueuePrompt(t)
	}
	testInteractive(t, "album name", searchResults, lookupRequest, lookupEntries, matcherFunc, steps, isAddQueue, selectPrompt)
}

func TestAddQueueAlbumSelectAll(t *testing.T) {
	matcherFunc := func(arg interface{}) bool {
		req := arg.(*platune.AddToQueueRequest)
		return len(req.Songs) == 2 && req.Songs[0] == "/test/path/1" && req.Songs[1] == "/test/path/2"
	}
	testAlbumSelectAll(t, matcherFunc, addQueueCmdText+" ", true, true)
	testAlbumSelectAll(t, matcherFunc, addQueueCmdText+" ", true, false)
}

func TestSetQueueAlbumSelectAll(t *testing.T) {
	matcherFunc := func(arg interface{}) bool {
		req := arg.(*platune.QueueRequest)
		return len(req.Queue) == 2 && req.Queue[0] == "/test/path/1" && req.Queue[1] == "/test/path/2"
	}
	testAlbumSelectAll(t, matcherFunc, "", false, true)
	testAlbumSelectAll(t, matcherFunc, "", false, false)
}

func testArtistSelection(t *testing.T, matcherFunc func(arg interface{}) bool, prefix string, isAddQueue bool, selectPrompt bool) {
	searchResults := []*platune.SearchResult{
		{Entry: "artist name", EntryType: platune.EntryType_ARTIST, CorrelationIds: []int32{1}, Description: "artist desc"},
	}
	lookupRequest := &platune.LookupRequest{EntryType: platune.EntryType_ARTIST, CorrelationIds: []int32{1}}
	lookupEntries := []*platune.LookupEntry{
		{Artist: "artist name", Album: "album 1", Song: "track 1", Path: "/test/path/1", Track: 1},
		{Artist: "artist name", Album: "album 1", Song: "track 2", Path: "/test/path/2", Track: 1},
		{Artist: "artist name", Album: "album 2", Song: "track 1", Path: "/test/path/3", Track: 1},
		{Artist: "artist name", Album: "album 2", Song: "track 2", Path: "/test/path/4", Track: 1},
	}

	steps := []completionCase{
		{in: prefix + "artist name", outLength: 1, choiceText: "artist name", choiceIndex: 0},
		{in: "album 1", outLength: 1, choiceText: "album 1", choiceIndex: 0},
		{in: "track 1", outLength: 1, choiceText: "track 1", choiceIndex: 0},
	}
	initState()
	if !isAddQueue {
		initSetQueuePrompt(t)
	}
	testInteractive(t, "artist name", searchResults, lookupRequest, lookupEntries, matcherFunc, steps, isAddQueue, selectPrompt)
}

func TestAddQueueArtistSelection(t *testing.T) {
	matcherFunc := func(arg interface{}) bool {
		req := arg.(*platune.AddToQueueRequest)
		return len(req.Songs) == 1 && req.Songs[0] == "/test/path/1"
	}
	testArtistSelection(t, matcherFunc, addQueueCmdText+" ", true, true)
	testArtistSelection(t, matcherFunc, addQueueCmdText+" ", true, false)
}

func TestSetQueueArtistSelection(t *testing.T) {
	matcherFunc := func(arg interface{}) bool {
		req := arg.(*platune.QueueRequest)
		return len(req.Queue) == 1 && req.Queue[0] == "/test/path/1"
	}
	testArtistSelection(t, matcherFunc, "", false, true)
	testArtistSelection(t, matcherFunc, "", false, false)
}

func testArtistSelectAll(t *testing.T, matcherFunc func(arg interface{}) bool, prefix string, isAddQueue bool, selectPrompt bool) {
	searchResults := []*platune.SearchResult{
		{Entry: "artist name", EntryType: platune.EntryType_ARTIST, CorrelationIds: []int32{1}, Description: "artist desc"},
	}
	lookupRequest := &platune.LookupRequest{EntryType: platune.EntryType_ARTIST, CorrelationIds: []int32{1}}
	lookupEntries := []*platune.LookupEntry{
		{Artist: "artist name", Album: "album 1", Song: "track 1", Path: "/test/path/1", Track: 1},
		{Artist: "artist name", Album: "album 1", Song: "track 2", Path: "/test/path/2", Track: 1},
		{Artist: "artist name", Album: "album 2", Song: "track 1", Path: "/test/path/3", Track: 1},
		{Artist: "artist name", Album: "album 2", Song: "track 2", Path: "/test/path/4", Track: 1},
	}

	steps := []completionCase{
		{in: prefix + "artist name", outLength: 1, choiceText: "artist name", choiceIndex: 0},
		{in: selectAll, outLength: 1, choiceText: selectAll, choiceIndex: 0},
	}
	initState()
	if !isAddQueue {
		initSetQueuePrompt(t)
	}
	testInteractive(t, "artist name", searchResults, lookupRequest, lookupEntries, matcherFunc, steps, isAddQueue, selectPrompt)
}

func TestAddQueueArtistSelectAll(t *testing.T) {
	matcherFunc := func(arg interface{}) bool {
		req := arg.(*platune.AddToQueueRequest)
		return len(req.Songs) == 4 &&
			req.Songs[0] == "/test/path/1" &&
			req.Songs[1] == "/test/path/2" &&
			req.Songs[2] == "/test/path/3" &&
			req.Songs[3] == "/test/path/4"
	}
	testArtistSelectAll(t, matcherFunc, addQueueCmdText+" ", true, true)
	testArtistSelectAll(t, matcherFunc, addQueueCmdText+" ", true, false)
}

func TestSetQueueArtistSelectAll(t *testing.T) {
	matcherFunc := func(arg interface{}) bool {
		req := arg.(*platune.QueueRequest)
		return len(req.Queue) == 4 &&
			req.Queue[0] == "/test/path/1" &&
			req.Queue[1] == "/test/path/2" &&
			req.Queue[2] == "/test/path/3" &&
			req.Queue[3] == "/test/path/4"
	}
	testArtistSelectAll(t, matcherFunc, "", false, true)
	testArtistSelectAll(t, matcherFunc, "", false, false)
}

func testArtistSelectOneAlbum(t *testing.T, matcherFunc func(arg interface{}) bool, prefix string, isAddQueue bool, selectPrompt bool) {
	searchResults := []*platune.SearchResult{
		{Entry: "artist name", EntryType: platune.EntryType_ARTIST, CorrelationIds: []int32{1}, Description: "artist desc"},
	}
	lookupRequest := &platune.LookupRequest{EntryType: platune.EntryType_ARTIST, CorrelationIds: []int32{1}}
	lookupEntries := []*platune.LookupEntry{
		{Artist: "artist name", Album: "album 1", Song: "track 1", Path: "/test/path/1", Track: 1},
		{Artist: "artist name", Album: "album 1", Song: "track 2", Path: "/test/path/2", Track: 1},
		{Artist: "artist name", Album: "album 2", Song: "track 1", Path: "/test/path/3", Track: 1},
		{Artist: "artist name", Album: "album 2", Song: "track 2", Path: "/test/path/4", Track: 1},
	}

	steps := []completionCase{
		{in: prefix + "artist name", outLength: 1, choiceText: "artist name", choiceIndex: 0},
		{in: "album 2", outLength: 1, choiceText: "album 2", choiceIndex: 0},
		{in: selectAll, outLength: 1, choiceText: selectAll, choiceIndex: 0},
	}
	initState()
	if !isAddQueue {
		initSetQueuePrompt(t)
	}
	testInteractive(t, "artist name", searchResults, lookupRequest, lookupEntries, matcherFunc, steps, isAddQueue, selectPrompt)
}

func TestAddQueueArtistSelectOneAlbum(t *testing.T) {
	matcherFunc := func(arg interface{}) bool {
		req := arg.(*platune.AddToQueueRequest)
		return len(req.Songs) == 2 &&
			req.Songs[0] == "/test/path/3" &&
			req.Songs[1] == "/test/path/4"
	}
	testArtistSelectOneAlbum(t, matcherFunc, addQueueCmdText+" ", true, true)
	testArtistSelectOneAlbum(t, matcherFunc, addQueueCmdText+" ", true, false)
}

func TestSetQueueArtistSelectOneAlbum(t *testing.T) {
	matcherFunc := func(arg interface{}) bool {
		req := arg.(*platune.QueueRequest)
		return len(req.Queue) == 2 &&
			req.Queue[0] == "/test/path/3" &&
			req.Queue[1] == "/test/path/4"
	}
	testArtistSelectOneAlbum(t, matcherFunc, "", false, true)
	testArtistSelectOneAlbum(t, matcherFunc, "", false, false)
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

func TestSetQueueExecutor(t *testing.T) {
	initState()
	state.executor(setQueueCmdText, nil)
	if state.mode[0] != setQueueCmdText+"> " {
		t.Error(fmt.Sprintf("Live prefix should be set to %s> ", setQueueCmdText))
	}
	state.executor("root.go", &prompt.Suggest{Text: "root.go", Metadata: "root.go"})
	if len(state.currentQueue) != 1 {
		t.Error("Should've added an item to the queue")
	}
	if !strings.HasSuffix(state.currentQueue[0].Path, "root.go") {
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
