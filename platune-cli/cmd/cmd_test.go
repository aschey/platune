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

func TestAddQueueDbCompleter(t *testing.T) {
	searchClient = nil
	buf := prompt.NewBuffer()
	buf.InsertText(addQueueCmdText+" song name", false, true)
	doc := buf.Document()

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	mock := test.NewMockManagementClient(ctrl)
	stream := test.NewMockManagement_SearchClient(ctrl)
	stream.EXPECT().Send(gomock.Any()).Return(nil)
	artist := "blah"
	stream.EXPECT().Recv().Return(&platune.SearchResponse{Results: []*platune.SearchResult{
		{Entry: "song name", EntryType: platune.EntryType_SONG, Artist: &artist, CorrelationIds: []int32{1}, Description: "song desc"},
	}}, nil)

	mock.EXPECT().Search(gomock.Any()).Return(stream, nil)
	internal.Client = internal.NewTestClient(nil, mock)
	initState()
	results := state.completer(*doc)
	if len(results) != 1 {
		t.Error("Should've found one result")
	}
	if results[0].Text != "song name" {
		t.Error("Result should be 'song name'")
	}
	if results[0].Description != "song desc" {
		t.Error("Description should be 'song desc'")
	}
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
