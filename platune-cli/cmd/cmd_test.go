package cmd

import (
	"context"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/MarvinJWendt/testza"
	"github.com/aschey/go-prompt"
	"github.com/aschey/platune/cli/v2/internal"
	"github.com/aschey/platune/cli/v2/internal/deleted"
	"github.com/aschey/platune/cli/v2/internal/mode"
	"github.com/aschey/platune/cli/v2/internal/search"
	"github.com/aschey/platune/cli/v2/internal/statusbar"
	"github.com/aschey/platune/cli/v2/test"
	management_v1 "github.com/aschey/platune/client/management_v1"
	player_v1 "github.com/aschey/platune/client/player_v1"
	"go.uber.org/mock/gomock"
	"google.golang.org/protobuf/types/known/emptypb"
)

type completionCase struct {
	in          string
	outLength   int
	choiceIndex int
	choiceText  string
}

var originalArgs = os.Args

func runPlayerTest(t *testing.T, expected string,
	expectFunc func(expect *test.MockPlayerClientMockRecorder), args ...string,
) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	playerMock := test.NewMockPlayerClient(ctrl)
	mgmtMock := test.NewMockManagementClient(ctrl)

	expectFunc(playerMock.EXPECT())
	client := internal.NewTestClient(playerMock, mgmtMock)

	runTest(t, expected, &client, args...)
}

func runManagementTest(t *testing.T, expected string,
	expectFunc func(expect *test.MockManagementClientMockRecorder), args ...string,
) string {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	mock := test.NewMockManagementClient(ctrl)

	expectFunc(mock.EXPECT())
	client := internal.NewTestClient(nil, mock)

	return runTest(t, expected, &client, args...)
}

func runTest(t *testing.T, expected string, client *internal.PlatuneClient, args ...string) string {
	os.Args = append(originalArgs, args...)
	rootCmd := newRootCmd()

	ctx := context.Background()
	deleted := deleted.NewDeleted(client)
	search := search.NewSearch(client)

	state := NewState(client, deleted, make(statusbar.StatusChan), nil)

	outStr, err := testza.CaptureStdout(func(out io.Writer) error {
		return start(rootCmd, ctx, client, state, deleted, search)
	})

	testza.AssertNoError(t, err)
	testza.AssertTrue(t, expected == "" || outStr == expected,
		fmt.Sprintf("Expected %s, Got %s", expected, outStr))

	return outStr
}

func initSetQueuePrompt(t *testing.T, state *cmdState) {
	state.executor(setQueueCmdText, nil, []prompt.Suggest{})
	testza.AssertEqual(t, mode.SetQueueMode, state.mode.First())
}

func executeInteractive(t *testing.T, state *cmdState, steps []completionCase, selectPrompt bool) {
	for _, step := range steps {
		buf := prompt.NewBuffer()
		buf.InsertText(step.in, false, true)
		doc := buf.Document()
		resultChan := make(chan []prompt.Suggest, 1)
		state.completer(*doc, resultChan)
		results := <-resultChan
		testza.AssertLen(t, results, step.outLength)

		choice := results[step.choiceIndex]
		testza.AssertEqual(t, step.choiceText, choice.Text)

		if selectPrompt {
			state.executor(step.in, &results[step.choiceIndex], results)
		} else {
			state.executor(step.in, nil, results)
		}
	}
}

func testInteractive(
	t *testing.T,
	searchQuery string,
	searchResults []*management_v1.SearchResult,
	lookupRequest *management_v1.LookupRequest,
	lookupEntries []*management_v1.LookupEntry,
	matcherFunc func(arg interface{}) bool,
	steps []completionCase,
	isAddQueue bool,
	selectPrompt bool,
) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	mgmtMock := test.NewMockManagementClient(ctrl)
	playerMock := test.NewMockPlayerClient(ctrl)
	stream := test.NewMockBidiStreamingClient[management_v1.SearchRequest, management_v1.SearchResponse](ctrl)
	stream.EXPECT().Send(&management_v1.SearchRequest{Query: searchQuery}).Return(nil)

	stream.EXPECT().Recv().Return(&management_v1.SearchResponse{Results: searchResults}, nil)

	mgmtMock.EXPECT().Search(gomock.Any()).Return(stream, nil)
	mgmtMock.EXPECT().
		Lookup(gomock.Any(), lookupRequest).
		Return(&management_v1.LookupResponse{Entries: lookupEntries}, nil)
	matcher := test.NewMatcher(func(arg interface{}) bool {
		return matcherFunc(arg)
	})
	if isAddQueue {
		playerMock.EXPECT().AddToQueue(gomock.Any(), matcher)
	} else {
		playerMock.EXPECT().SetQueue(gomock.Any(), matcher)
	}

	client := internal.NewTestClient(playerMock, mgmtMock)
	deleted := deleted.NewDeleted(&client)
	state := NewState(&client, deleted, make(statusbar.StatusChan), nil)

	if !isAddQueue {
		initSetQueuePrompt(t, state)
	}

	executeInteractive(t, state, steps, selectPrompt)

	if !isAddQueue {
		state.executor("", nil, []prompt.Suggest{})
	}
}

func TestAddQueueFile(t *testing.T) {
	testSong := "root.go"
	runPlayerTest(t, "Added\n", func(expect *test.MockPlayerClientMockRecorder) {
		matcher := test.NewMatcher(func(arg interface{}) bool {
			path, _ := filepath.Abs(testSong)
			return arg.(*player_v1.AddToQueueRequest).Songs[0] == path
		})
		expect.AddToQueue(gomock.Any(), matcher)
	}, addQueueCmdText, testSong)
}

func TestAddQueueUrl(t *testing.T) {
	testSong := "http://test.com/blah.mp3"
	runPlayerTest(t, "Added\n", func(expect *test.MockPlayerClientMockRecorder) {
		matcher := test.NewMatcher(func(arg interface{}) bool {
			return arg.(*player_v1.AddToQueueRequest).Songs[0] == testSong
		})
		expect.AddToQueue(gomock.Any(), matcher)
	}, addQueueCmdText, testSong)
}

func TestSetQueueFile(t *testing.T) {
	testSong := "root.go"
	runPlayerTest(t, "Queue Set\n", func(expect *test.MockPlayerClientMockRecorder) {
		matcher := test.NewMatcher(func(arg interface{}) bool {
			queue := arg.(*player_v1.QueueRequest).Queue
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
			queue := arg.(*player_v1.QueueRequest).Queue
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
			return arg.(*player_v1.SeekRequest).Time.AsDuration() == time.Duration(
				tc.expected*uint64(time.Millisecond),
			)
		})
		runPlayerTest(
			t,
			fmt.Sprintf("Seeked to %s\n", tc.formatStr),
			func(expect *test.MockPlayerClientMockRecorder) {
				expect.Seek(gomock.Any(), matcher)
			},
			seekCmdText,
			tc.formatStr,
		)
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
		stream := test.NewMockServerStreamingClient[management_v1.Progress](ctrl)
		stream.EXPECT().
			Recv().
			Return(&management_v1.Progress{Percentage: 0.1, Finished: false, Job: "sync"}, nil)
		stream.EXPECT().
			Recv().
			Return(&management_v1.Progress{Percentage: 0.1, Finished: false, Job: "somethingElse"}, nil)
		// subscriber channel runs in a separate goroutine so we don't know how many times it will execute before the main thread finishes
		stream.EXPECT().
			Recv().
			Return(&management_v1.Progress{Percentage: 1.0, Finished: true, Job: "sync"}, nil).
			MinTimes(1)
		expect.SubscribeEvents(gomock.Any(), gomock.Any()).Return(stream, nil)
		expect.StartSync(gomock.Any(), &emptypb.Empty{}).Return(&emptypb.Empty{}, nil)

		expect.GetDeleted(gomock.Any(), gomock.Any()).Return(&management_v1.GetDeletedResponse{
			Results: []*management_v1.DeletedResult{},
		}, nil)
	}, syncCmdText)

	testza.AssertGreater(t, len(res), 0)
}

func TestGetAllFolders(t *testing.T) {
	response := "C://test"
	res := runManagementTest(t, "", func(expect *test.MockManagementClientMockRecorder) {
		expect.GetAllFolders(gomock.Any(), gomock.Any()).
			Return(&management_v1.FoldersMessage{Folders: []string{response}}, nil)
	}, getAllFoldersCmdText)

	testza.AssertContains(t, res, response)
}

func TestAddFolder(t *testing.T) {
	folder := "folder1"
	runManagementTest(t, "Added\n", func(expect *test.MockManagementClientMockRecorder) {
		matcher := test.NewMatcher(func(arg interface{}) bool {
			folders := arg.(*management_v1.FoldersMessage).Folders
			return folders[0] == folder
		})
		expect.AddFolders(gomock.Any(), matcher)
	}, addFolderCmdText, folder)
}

func TestSetMount(t *testing.T) {
	folder := "/home/test"
	runManagementTest(t, "Set\n", func(expect *test.MockManagementClientMockRecorder) {
		matcher := test.NewMatcher(func(arg interface{}) bool {
			mount := arg.(*management_v1.RegisteredMountMessage).Mount
			return mount == folder
		})
		expect.RegisterMount(gomock.Any(), matcher)
	}, setMountCmdText, folder)
}

func TestSetVolume(t *testing.T) {
	volume := float32(0.5)
	runPlayerTest(t, "Set\n", func(expect *test.MockPlayerClientMockRecorder) {
		matcher := test.NewMatcher(func(arg interface{}) bool {
			volumeArg := arg.(*player_v1.SetVolumeRequest).Volume
			return volumeArg == volume
		})
		expect.SetVolume(gomock.Any(), matcher)
	}, setVolumeCmdText, fmt.Sprintf("%f", volume))
}

func testFileCompleter(t *testing.T, prefix string, isAddQueue bool) {
	// initState()
	// searchClient = nil
	buf := prompt.NewBuffer()
	buf.InsertText(prefix+"root", false, true)
	doc := buf.Document()

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	mock := test.NewMockManagementClient(ctrl)
	stream := test.NewMockBidiStreamingClient[management_v1.SearchRequest, management_v1.SearchResponse](ctrl)
	stream.EXPECT().Send(gomock.Any()).Return(nil)
	stream.EXPECT().Recv().Return(&management_v1.SearchResponse{Results: []*management_v1.SearchResult{}}, nil)

	mock.EXPECT().Search(gomock.Any()).Return(stream, nil)
	client := internal.NewTestClient(nil, mock)
	deleted := deleted.NewDeleted(&client)
	state := NewState(&client, deleted, make(statusbar.StatusChan), nil)

	if !isAddQueue {
		initSetQueuePrompt(t, state)
	}

	resultChan := make(chan []prompt.Suggest, 1)
	state.completer(*doc, resultChan)
	results := <-resultChan

	testza.AssertLen(t, results, 1)
	testza.AssertEqual(t, "root.go", results[0].Text)
}

func TestSetQueueFileCompleter(t *testing.T) {
	testFileCompleter(t, "", false)
}

func TestAddQueueFileCompleter(t *testing.T) {
	testFileCompleter(t, addQueueCmdText+" ", true)
}

func testSongSelection(
	t *testing.T,
	matcherFunc func(arg interface{}) bool,
	prefix string,
	isAddQueue bool,
	selectPrompt bool,
) {
	artist := "blah"
	searchResults := []*management_v1.SearchResult{
		{
			Entry:          "song name",
			EntryType:      management_v1.EntryType_SONG,
			Artist:         &artist,
			CorrelationIds: []int64{1},
			Description:    "song desc",
		},
	}
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

	steps := []completionCase{
		{in: prefix + "song name", outLength: 1, choiceText: "song name", choiceIndex: 0},
	}

	testInteractive(
		t,
		"song name",
		searchResults,
		lookupRequest,
		lookupEntries,
		matcherFunc,
		steps,
		isAddQueue,
		selectPrompt,
	)
}

func TestAddQueueSongSelection(t *testing.T) {
	matcherFunc := func(arg interface{}) bool {
		return arg.(*player_v1.AddToQueueRequest).Songs[0] == "/test/path/1"
	}
	testSongSelection(t, matcherFunc, addQueueCmdText+" ", true, true)
	testSongSelection(t, matcherFunc, addQueueCmdText+" ", true, false)
}

func TestSetQueueSongSelection(t *testing.T) {
	matcherFunc := func(arg interface{}) bool {
		return arg.(*player_v1.QueueRequest).Queue[0] == "/test/path/1"
	}
	testSongSelection(t, matcherFunc, "", false, true)
	testSongSelection(t, matcherFunc, "", false, false)
}

func testAlbumSelection(
	t *testing.T,
	matcherFunc func(arg interface{}) bool,
	prefix string,
	isAddQueue bool,
	selectPrompt bool,
) {
	artist := "blah"
	searchResults := []*management_v1.SearchResult{
		{
			Entry:          "album name",
			EntryType:      management_v1.EntryType_ALBUM,
			Artist:         &artist,
			CorrelationIds: []int64{1},
			Description:    "album desc",
		},
	}
	lookupRequest := &management_v1.LookupRequest{
		EntryType:      management_v1.EntryType_ALBUM,
		CorrelationIds: []int64{1},
	}
	lookupEntries := []*management_v1.LookupEntry{
		{
			Artist: "artist name",
			Album:  "album name",
			Song:   "track 1",
			Path:   "/test/path/1",
			Track:  1,
		},
		{
			Artist: "artist name",
			Album:  "album name",
			Song:   "track 2",
			Path:   "/test/path/2",
			Track:  2,
		},
	}

	steps := []completionCase{
		{in: prefix + "album name", outLength: 1, choiceText: "album name", choiceIndex: 0},
		{in: "track 1", outLength: 1, choiceText: "track 1", choiceIndex: 0},
	}

	testInteractive(
		t,
		"album name",
		searchResults,
		lookupRequest,
		lookupEntries,
		matcherFunc,
		steps,
		isAddQueue,
		selectPrompt,
	)
}

func TestAddQueueAlbumSelection(t *testing.T) {
	matcherFunc := func(arg interface{}) bool {
		req := arg.(*player_v1.AddToQueueRequest)
		return len(req.Songs) == 1 && req.Songs[0] == "/test/path/1"
	}
	testAlbumSelection(t, matcherFunc, addQueueCmdText+" ", true, true)
	testAlbumSelection(t, matcherFunc, addQueueCmdText+" ", true, false)
}

func TestSetQueueAlbumSelection(t *testing.T) {
	matcherFunc := func(arg interface{}) bool {
		req := arg.(*player_v1.QueueRequest)
		return len(req.Queue) == 1 && req.Queue[0] == "/test/path/1"
	}
	testAlbumSelection(t, matcherFunc, "", false, true)
	testAlbumSelection(t, matcherFunc, "", false, false)
}

func testAlbumSelectAll(
	t *testing.T,
	matcherFunc func(arg interface{}) bool,
	prefix string,
	isAddQueue bool,
	selectPrompt bool,
) {
	artist := "blah"
	searchResults := []*management_v1.SearchResult{
		{
			Entry:          "album name",
			EntryType:      management_v1.EntryType_ALBUM,
			Artist:         &artist,
			CorrelationIds: []int64{1},
			Description:    "album desc",
		},
	}
	lookupRequest := &management_v1.LookupRequest{
		EntryType:      management_v1.EntryType_ALBUM,
		CorrelationIds: []int64{1},
	}
	lookupEntries := []*management_v1.LookupEntry{
		{
			Artist: "artist name",
			Album:  "album name",
			Song:   "track 1",
			Path:   "/test/path/1",
			Track:  1,
		},
		{
			Artist: "artist name",
			Album:  "album name",
			Song:   "track 2",
			Path:   "/test/path/2",
			Track:  2,
		},
	}

	steps := []completionCase{
		{in: prefix + "album name", outLength: 1, choiceText: "album name", choiceIndex: 0},
		{in: selectAll, outLength: 1, choiceText: selectAll, choiceIndex: 0},
	}

	testInteractive(
		t,
		"album name",
		searchResults,
		lookupRequest,
		lookupEntries,
		matcherFunc,
		steps,
		isAddQueue,
		selectPrompt,
	)
}

func TestAddQueueAlbumSelectAll(t *testing.T) {
	matcherFunc := func(arg interface{}) bool {
		req := arg.(*player_v1.AddToQueueRequest)
		return len(req.Songs) == 2 && req.Songs[0] == "/test/path/1" &&
			req.Songs[1] == "/test/path/2"
	}
	testAlbumSelectAll(t, matcherFunc, addQueueCmdText+" ", true, true)
	testAlbumSelectAll(t, matcherFunc, addQueueCmdText+" ", true, false)
}

func TestSetQueueAlbumSelectAll(t *testing.T) {
	matcherFunc := func(arg interface{}) bool {
		req := arg.(*player_v1.QueueRequest)
		return len(req.Queue) == 2 && req.Queue[0] == "/test/path/1" &&
			req.Queue[1] == "/test/path/2"
	}
	testAlbumSelectAll(t, matcherFunc, "", false, true)
	testAlbumSelectAll(t, matcherFunc, "", false, false)
}

func testArtistSelection(
	t *testing.T,
	matcherFunc func(arg interface{}) bool,
	prefix string,
	isAddQueue bool,
	selectPrompt bool,
) {
	searchResults := []*management_v1.SearchResult{
		{
			Entry:          "artist name",
			EntryType:      management_v1.EntryType_ARTIST,
			CorrelationIds: []int64{1},
			Description:    "artist desc",
		},
	}
	lookupRequest := &management_v1.LookupRequest{
		EntryType:      management_v1.EntryType_ARTIST,
		CorrelationIds: []int64{1},
	}
	lookupEntries := []*management_v1.LookupEntry{
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

	testInteractive(
		t,
		"artist name",
		searchResults,
		lookupRequest,
		lookupEntries,
		matcherFunc,
		steps,
		isAddQueue,
		selectPrompt,
	)
}

func TestAddQueueArtistSelection(t *testing.T) {
	matcherFunc := func(arg interface{}) bool {
		req := arg.(*player_v1.AddToQueueRequest)
		return len(req.Songs) == 1 && req.Songs[0] == "/test/path/1"
	}
	testArtistSelection(t, matcherFunc, addQueueCmdText+" ", true, true)
	testArtistSelection(t, matcherFunc, addQueueCmdText+" ", true, false)
}

func TestSetQueueArtistSelection(t *testing.T) {
	matcherFunc := func(arg interface{}) bool {
		req := arg.(*player_v1.QueueRequest)
		return len(req.Queue) == 1 && req.Queue[0] == "/test/path/1"
	}
	testArtistSelection(t, matcherFunc, "", false, true)
	testArtistSelection(t, matcherFunc, "", false, false)
}

func testArtistSelectAll(
	t *testing.T,
	matcherFunc func(arg interface{}) bool,
	prefix string,
	isAddQueue bool,
	selectPrompt bool,
) {
	searchResults := []*management_v1.SearchResult{
		{
			Entry:          "artist name",
			EntryType:      management_v1.EntryType_ARTIST,
			CorrelationIds: []int64{1},
			Description:    "artist desc",
		},
	}
	lookupRequest := &management_v1.LookupRequest{
		EntryType:      management_v1.EntryType_ARTIST,
		CorrelationIds: []int64{1},
	}
	lookupEntries := []*management_v1.LookupEntry{
		{Artist: "artist name", Album: "album 1", Song: "track 1", Path: "/test/path/1", Track: 1},
		{Artist: "artist name", Album: "album 1", Song: "track 2", Path: "/test/path/2", Track: 1},
		{Artist: "artist name", Album: "album 2", Song: "track 1", Path: "/test/path/3", Track: 1},
		{Artist: "artist name", Album: "album 2", Song: "track 2", Path: "/test/path/4", Track: 1},
	}

	steps := []completionCase{
		{in: prefix + "artist name", outLength: 1, choiceText: "artist name", choiceIndex: 0},
		{in: selectAll, outLength: 1, choiceText: selectAll, choiceIndex: 0},
	}

	testInteractive(
		t,
		"artist name",
		searchResults,
		lookupRequest,
		lookupEntries,
		matcherFunc,
		steps,
		isAddQueue,
		selectPrompt,
	)
}

func TestAddQueueArtistSelectAll(t *testing.T) {
	matcherFunc := func(arg interface{}) bool {
		req := arg.(*player_v1.AddToQueueRequest)
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
		req := arg.(*player_v1.QueueRequest)
		return len(req.Queue) == 4 &&
			req.Queue[0] == "/test/path/1" &&
			req.Queue[1] == "/test/path/2" &&
			req.Queue[2] == "/test/path/3" &&
			req.Queue[3] == "/test/path/4"
	}
	testArtistSelectAll(t, matcherFunc, "", false, true)
	testArtistSelectAll(t, matcherFunc, "", false, false)
}

func testArtistSelectOneAlbum(
	t *testing.T,
	matcherFunc func(arg interface{}) bool,
	prefix string,
	isAddQueue bool,
	selectPrompt bool,
) {
	searchResults := []*management_v1.SearchResult{
		{
			Entry:          "artist name",
			EntryType:      management_v1.EntryType_ARTIST,
			CorrelationIds: []int64{1},
			Description:    "artist desc",
		},
	}
	lookupRequest := &management_v1.LookupRequest{
		EntryType:      management_v1.EntryType_ARTIST,
		CorrelationIds: []int64{1},
	}
	lookupEntries := []*management_v1.LookupEntry{
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

	testInteractive(
		t,
		"artist name",
		searchResults,
		lookupRequest,
		lookupEntries,
		matcherFunc,
		steps,
		isAddQueue,
		selectPrompt,
	)
}

func TestAddQueueArtistSelectOneAlbum(t *testing.T) {
	matcherFunc := func(arg interface{}) bool {
		req := arg.(*player_v1.AddToQueueRequest)
		return len(req.Songs) == 2 &&
			req.Songs[0] == "/test/path/3" &&
			req.Songs[1] == "/test/path/4"
	}
	testArtistSelectOneAlbum(t, matcherFunc, addQueueCmdText+" ", true, true)
	testArtistSelectOneAlbum(t, matcherFunc, addQueueCmdText+" ", true, false)
}

func TestSetQueueArtistSelectOneAlbum(t *testing.T) {
	matcherFunc := func(arg interface{}) bool {
		req := arg.(*player_v1.QueueRequest)
		return len(req.Queue) == 2 &&
			req.Queue[0] == "/test/path/3" &&
			req.Queue[1] == "/test/path/4"
	}
	testArtistSelectOneAlbum(t, matcherFunc, "", false, true)
	testArtistSelectOneAlbum(t, matcherFunc, "", false, false)
}

func TestAddFolderCompleter(t *testing.T) {
	buf := prompt.NewBuffer()
	buf.InsertText(addFolderCmdText+" root", false, true)
	doc := buf.Document()

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	playerClient := test.NewMockPlayerClient(ctrl)
	mgmtClient := test.NewMockManagementClient(ctrl)

	client := internal.NewTestClient(playerClient, mgmtClient)
	deleted := deleted.NewDeleted(&client)
	state := NewState(&client, deleted, make(statusbar.StatusChan), nil)

	resultChan := make(chan []prompt.Suggest, 1)
	state.completer(*doc, resultChan)
	results := <-resultChan
	testza.AssertLen(t, results, 1)
	testza.AssertEqual(t, "root.go", results[0].Text)
}

func TestSetQueueExecutor(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	playerMock := test.NewMockPlayerClient(ctrl)
	mgmtMock := test.NewMockManagementClient(ctrl)
	path := "/test/path/1"

	playerMock.EXPECT().SetQueue(gomock.Any(), &player_v1.QueueRequest{Queue: []string{path, path}})

	entries := []*management_v1.LookupEntry{
		{Artist: "artist name", Album: "album 1", Song: "song name", Path: path, Track: 1},
	}
	mgmtMock.EXPECT().Lookup(gomock.Any(), &management_v1.LookupRequest{
		EntryType:      management_v1.EntryType_SONG,
		CorrelationIds: []int64{1},
	}).
		Return(&management_v1.LookupResponse{Entries: entries}, nil).Times(2)

	client := internal.NewTestClient(playerMock, mgmtMock)
	deleted := deleted.NewDeleted(&client)
	state := NewState(&client, deleted, make(statusbar.StatusChan), nil)

	state.executor(setQueueCmdText, nil, []prompt.Suggest{})
	testza.AssertEqual(t, mode.SetQueueMode, state.mode.First())

	suggests := []prompt.Suggest{{Text: "song name", Metadata: &management_v1.SearchResult{
		EntryType:      management_v1.EntryType_SONG,
		CorrelationIds: []int64{1},
	}}}

	state.executor("song name", &suggests[0], suggests)
	testza.AssertEqual(t, mode.SetQueueMode, state.mode.First())
	testza.AssertLen(t, state.currentQueue, 1)

	state.executor("song name", &suggests[0], suggests)
	testza.AssertEqual(t, mode.SetQueueMode, state.mode.First())
	testza.AssertLen(t, state.currentQueue, 2)

	testza.AssertEqual(t, path, state.currentQueue[0].Path)
	testza.AssertEqual(t, path, state.currentQueue[1].Path)

	state.executor("", nil, []prompt.Suggest{})
	testza.AssertEqual(t, mode.NormalMode, state.mode.First())
}

func TestSetQueueExecutorFile(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	playerMock := test.NewMockPlayerClient(ctrl)
	matcher := test.NewMatcher(func(arg interface{}) bool {
		queue := arg.(*player_v1.QueueRequest).Queue
		return strings.HasSuffix(queue[0], "root.go") && strings.HasSuffix(queue[1], "root.go")
	})
	playerMock.EXPECT().SetQueue(gomock.Any(), matcher)

	mgmtMock := test.NewMockManagementClient(ctrl)
	client := internal.NewTestClient(playerMock, mgmtMock)
	deleted := deleted.NewDeleted(&client)
	state := NewState(&client, deleted, make(statusbar.StatusChan), nil)

	state.executor(setQueueCmdText, nil, []prompt.Suggest{})
	testza.AssertEqual(t, mode.SetQueueMode, state.mode.First())

	suggests := []prompt.Suggest{{Text: "root.go", Metadata: "root.go"}}
	state.executor("root.go", &suggests[0], suggests)
	testza.AssertEqual(t, mode.SetQueueMode, state.mode.First())
	testza.AssertLen(t, state.currentQueue, 1)

	state.executor("root.go", &suggests[0], suggests)
	testza.AssertEqual(t, mode.SetQueueMode, state.mode.First())
	testza.AssertLen(t, state.currentQueue, 2)

	testza.AssertTrue(
		t,
		strings.HasSuffix(state.currentQueue[0].Path, "root.go"),
		"root.go should've been added to the queue",
	)
	state.executor("", nil, []prompt.Suggest{})
	testza.AssertEqual(t, mode.NormalMode, state.mode.First())
}

func TestSetQueueExecutorInvalidFile(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	mgmtMock := test.NewMockManagementClient(ctrl)
	client := internal.NewTestClient(nil, mgmtMock)
	deleted := deleted.NewDeleted(&client)
	state := NewState(&client, deleted, make(statusbar.StatusChan), nil)

	state.executor(setQueueCmdText, nil, []prompt.Suggest{})
	testza.AssertEqual(t, mode.SetQueueMode, state.mode.First())

	suggests := []prompt.Suggest{{Text: "blah.go", Metadata: "blah.go"}}
	state.executor("blah.go", &suggests[0], suggests)
	testza.AssertLen(t, state.currentQueue, 0)

	state.executor("", nil, []prompt.Suggest{})
}
