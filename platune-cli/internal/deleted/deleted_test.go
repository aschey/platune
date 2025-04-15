package deleted

import (
	"bytes"
	"fmt"
	"io"
	"regexp"
	"testing"

	"github.com/MarvinJWendt/testza"
	"github.com/aschey/platune/cli/v2/internal"
	"github.com/aschey/platune/cli/v2/test"
	platune "github.com/aschey/platune/client"
	"github.com/charmbracelet/bubbles/list"
	tea "github.com/charmbracelet/bubbletea"
	"go.uber.org/mock/gomock"
	"google.golang.org/protobuf/types/known/emptypb"
)

func testRenderItem(t *testing.T, index int, checked bool, expected string) {
	results := []*platune.DeletedResult{
		{Path: "/test/path/1", Id: 1},
		{Path: "/test/path/2", Id: 2},
	}
	items := getItems(results)

	d := itemDelegate{}
	l := list.NewModel(items, d, 0, 0)

	var buf bytes.Buffer
	selectedItem := items[index].(item)
	if checked {
		selectedItem.selected = true
	}
	d.Render(&buf, l, index, selectedItem)

	out := buf.String()
	testza.AssertEqual(t, expected, out, fmt.Sprintf("Expected %s, got %s", expected, out))
}

func sendKeys(ctrl *gomock.Controller, msgs []tea.KeyMsg, deletedIds []int64, m *model) model {
	results := []*platune.DeletedResult{
		{Path: "/test/path/1", Id: 1},
		{Path: "/test/path/2", Id: 2},
	}
	items := getItems(results)
	d := itemDelegate{}
	l := list.NewModel(items, d, 0, 0)

	if m == nil {
		mock := test.NewMockManagementClient(ctrl)
		if len(deletedIds) > 0 {
			mock.EXPECT().DeleteTracks(gomock.Any(), &platune.IdMessage{Ids: deletedIds})
		}
		client := internal.NewTestClient(nil, mock)

		m = &model{list: l, client: &client, showConfirmDialog: false, cancelChosen: false, quitText: ""}
	}

	for _, msg := range msgs {
		newModel, _ := m.Update(msg)
		mm := newModel.(model)
		m = &mm
	}

	return *m
}

func cleanupView(m model) string {
	view := m.View()
	extraText := regexp.MustCompile(`\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])|\n|│`)
	whitespace := regexp.MustCompile(`\s+`)
	// remove ansi escape sequences and some extra special characters
	view = string(extraText.ReplaceAll([]byte(view), []byte("")))
	// replace extra whitespace with a single whitespace character
	view = string(whitespace.ReplaceAll([]byte(view), []byte(" ")))

	return view
}

func TestNoRenderWhenNoResults(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	mock := test.NewMockManagementClient(ctrl)
	response := &platune.GetDeletedResponse{Results: []*platune.DeletedResult{}}
	mock.EXPECT().GetDeleted(gomock.Any(), &emptypb.Empty{}).Return(response, nil)
	client := internal.NewTestClient(nil, mock)
	deleted := NewDeleted(&client)
	out, _ := testza.CaptureStdout(func(io.Writer) error {
		deleted.RenderDeletedFiles()
		return nil
	})
	testza.AssertEqual(t, "", out)
}

func TestRenderSelected(t *testing.T) {
	expected := selectedItemStyle.Render("▶ ◯ /test/path/1")
	testRenderItem(t, 0, false, expected)
}

func TestRender(t *testing.T) {
	expected := itemStyle.Render("◯ /test/path/2")
	testRenderItem(t, 1, false, expected)
}

func TestRenderCheckedSelected(t *testing.T) {
	expected := selectedItemStyle.Render("▶ ◉ /test/path/1")
	testRenderItem(t, 0, true, expected)
}

func TestRenderChecked(t *testing.T) {
	expected := itemStyle.Render("◉ /test/path/2")
	testRenderItem(t, 1, true, expected)
}

func TestChooseResultSpaceKey(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	m := sendKeys(ctrl, []tea.KeyMsg{{Type: tea.KeySpace}}, []int64{}, nil)
	testza.AssertTrue(t, m.list.Items()[0].(item).selected)
}

func TestChooseResultWhitespace(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	m := sendKeys(ctrl, []tea.KeyMsg{{Type: tea.KeyRunes, Runes: []rune(" ")}}, []int64{}, nil)
	testza.AssertTrue(t, m.list.Items()[0].(item).selected)
}

func TestChooseAllResults(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	m := sendKeys(ctrl, []tea.KeyMsg{{Type: tea.KeyRunes, Runes: []rune("a")}}, []int64{}, nil)
	testza.AssertTrue(t, m.list.Items()[0].(item).selected)
	testza.AssertTrue(t, m.list.Items()[1].(item).selected)
}

func TestUnselectAllResults(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	m := sendKeys(ctrl, []tea.KeyMsg{
		{Type: tea.KeyRunes, Runes: []rune("a")},
		{Type: tea.KeyRunes, Runes: []rune("a")},
	}, []int64{}, nil)
	testza.AssertFalse(t, m.list.Items()[0].(item).selected)
	testza.AssertFalse(t, m.list.Items()[1].(item).selected)
}

func TestReselectAllResults(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	m := sendKeys(ctrl, []tea.KeyMsg{
		{Type: tea.KeyRunes, Runes: []rune("a")},
		{Type: tea.KeyRunes, Runes: []rune(" ")},
		{Type: tea.KeyRunes, Runes: []rune("a")},
	}, []int64{}, nil)
	testza.AssertTrue(t, m.list.Items()[0].(item).selected)
	testza.AssertTrue(t, m.list.Items()[1].(item).selected)
}

func TestChooseNoFilesToDelete(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	m := sendKeys(ctrl, []tea.KeyMsg{{Type: tea.KeyEnter}}, []int64{}, nil)
	testza.AssertEqual(t, quitTextStyle.Render("No Songs Deleted"), m.View())
}

func TestDeleteAllFiles(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	m := sendKeys(ctrl, []tea.KeyMsg{{Type: tea.KeyRunes, Runes: []rune("a")}, {Type: tea.KeyEnter}}, []int64{1, 2}, nil)
	view := cleanupView(m)
	testza.AssertContains(t, view, "Are you sure you want to permanently delete 2 song(s)?")

	m = sendKeys(ctrl, []tea.KeyMsg{{Type: tea.KeyEnter}}, []int64{}, &m)
	view = cleanupView(m)
	testza.AssertContains(t, view, "2 song(s) deleted")
}

func TestDeleteFirstFile(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	m := sendKeys(ctrl, []tea.KeyMsg{{Type: tea.KeyRunes, Runes: []rune(" ")}, {Type: tea.KeyEnter}}, []int64{1}, nil)
	view := cleanupView(m)
	testza.AssertContains(t, view, "Are you sure you want to permanently delete 1 song(s)?")

	m = sendKeys(ctrl, []tea.KeyMsg{{Type: tea.KeyEnter}}, []int64{}, &m)
	view = cleanupView(m)
	testza.AssertContains(t, view, "1 song(s) deleted")
}

func TestDeleteSecondFile(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	m := sendKeys(ctrl, []tea.KeyMsg{{Type: tea.KeyDown}, {Type: tea.KeyRunes, Runes: []rune(" ")}, {Type: tea.KeyEnter}}, []int64{2}, nil)
	view := cleanupView(m)

	testza.AssertContains(t, view, "Are you sure you want to permanently delete 1 song(s)?")

	m = sendKeys(ctrl, []tea.KeyMsg{{Type: tea.KeyEnter}}, []int64{}, &m)
	view = cleanupView(m)
	testza.AssertContains(t, view, "1 song(s) deleted")
}

func TestCancelDelete(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	m := sendKeys(ctrl, []tea.KeyMsg{{Type: tea.KeyRunes, Runes: []rune("a")}, {Type: tea.KeyEnter}}, []int64{}, nil)
	view := cleanupView(m)

	testza.AssertContains(t, view, "Are you sure you want to permanently delete 2 song(s)?")

	m = sendKeys(ctrl, []tea.KeyMsg{{Type: tea.KeyLeft}, {Type: tea.KeyEnter}}, []int64{}, &m)
	view = cleanupView(m)
	testza.AssertContains(t, view, "▶ ◉ /test/path/1")
}

func TestCancelThenDelete(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	m := sendKeys(ctrl, []tea.KeyMsg{{Type: tea.KeyRunes, Runes: []rune("a")}, {Type: tea.KeyEnter}}, []int64{1, 2}, nil)
	view := cleanupView(m)

	testza.AssertContains(t, view, "Are you sure you want to permanently delete 2 song(s)?")

	m = sendKeys(ctrl, []tea.KeyMsg{{Type: tea.KeyLeft}, {Type: tea.KeyEnter}}, []int64{}, &m)
	view = cleanupView(m)
	testza.AssertContains(t, view, "▶ ◉ /test/path/1")

	m = sendKeys(ctrl, []tea.KeyMsg{{Type: tea.KeyEnter}}, []int64{}, &m)
	view = cleanupView(m)
	testza.AssertContains(t, view, "Are you sure you want to permanently delete 2 song(s)?")

	m = sendKeys(ctrl, []tea.KeyMsg{{Type: tea.KeyLeft}, {Type: tea.KeyEnter}}, []int64{}, &m)
	view = cleanupView(m)
	testza.AssertContains(t, view, "2 song(s) deleted")
}
