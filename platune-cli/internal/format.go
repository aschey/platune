package internal

import (
	"fmt"
	"strconv"
	"strings"

	"github.com/aschey/bubbleprompt/executor"
	platune "github.com/aschey/platune/client"
	"github.com/charmbracelet/lipgloss"
)

func NewInfoModel(message string) executor.StringModel {
	return executor.NewStringModel(lipgloss.NewStyle().Foreground(lipgloss.Color("245")).Render(message))
}

func PrettyPrintList(list []string) string {
	formatted := []string{}
	numberStyle := lipgloss.NewStyle().Foreground(lipgloss.Color("242"))
	for i := 0; i < len(list); i++ {
		formatted = append(
			formatted,
			fmt.Sprintf("%s %s", numberStyle.Render(strconv.Itoa(i+1)+"."), list[i]),
		)
	}
	return strings.Join(formatted, "\n")
}

type SearchMetadata struct {
	Result *platune.SearchResult
}
