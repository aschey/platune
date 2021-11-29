package deleted

import (
	"bytes"
	"fmt"
	"io"
	"testing"

	"github.com/MarvinJWendt/testza"
	"github.com/aschey/platune/cli/v2/internal"
	"github.com/aschey/platune/cli/v2/test"
	platune "github.com/aschey/platune/client"
	"github.com/charmbracelet/bubbles/list"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/golang/mock/gomock"
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

func sendKeys(t *testing.T, msgs []tea.KeyMsg) model {
	results := []*platune.DeletedResult{
		{Path: "/test/path/1", Id: 1},
		{Path: "/test/path/2", Id: 2},
	}
	items := getItems(results)
	d := itemDelegate{}
	l := list.NewModel(items, d, 0, 0)

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	mock := test.NewMockManagementClient(ctrl)
	client := internal.NewTestClient(nil, mock)

	m := model{list: l, client: &client, showConfirmDialog: false, cancelChosen: false, quitText: ""}
	for _, msg := range msgs {
		newModel, _ := m.Update(msg)
		m = newModel.(model)
	}

	return m
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
	m := sendKeys(t, []tea.KeyMsg{{Type: tea.KeySpace}})
	testza.AssertTrue(t, m.list.Items()[0].(item).selected)
}

func TestChooseResultWhitespace(t *testing.T) {
	m := sendKeys(t, []tea.KeyMsg{{Type: tea.KeyRunes, Runes: []rune(" ")}})
	testza.AssertTrue(t, m.list.Items()[0].(item).selected)
}

func TestChooseAllResults(t *testing.T) {
	m := sendKeys(t, []tea.KeyMsg{{Type: tea.KeyRunes, Runes: []rune("a")}})
	testza.AssertTrue(t, m.list.Items()[0].(item).selected)
	testza.AssertTrue(t, m.list.Items()[1].(item).selected)
}

func TestUnselectAllResults(t *testing.T) {
	m := sendKeys(t, []tea.KeyMsg{
		{Type: tea.KeyRunes, Runes: []rune("a")},
		{Type: tea.KeyRunes, Runes: []rune("a")},
	})
	testza.AssertFalse(t, m.list.Items()[0].(item).selected)
	testza.AssertFalse(t, m.list.Items()[1].(item).selected)
}

func TestReselectAllResults(t *testing.T) {
	m := sendKeys(t, []tea.KeyMsg{
		{Type: tea.KeyRunes, Runes: []rune("a")},
		{Type: tea.KeyRunes, Runes: []rune(" ")},
		{Type: tea.KeyRunes, Runes: []rune("a")},
	})
	testza.AssertTrue(t, m.list.Items()[0].(item).selected)
	testza.AssertTrue(t, m.list.Items()[1].(item).selected)
}
