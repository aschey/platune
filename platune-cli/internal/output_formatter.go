package internal

import (
	"fmt"
	"io/ioutil"
	"os"
	"strconv"
	"strings"

	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
	"github.com/spf13/pflag"
)

func addColor(replaceStr string, searchStr string, style lipgloss.Style) string {
	return strings.Replace(replaceStr, searchStr, style.Render(searchStr), -1)
}

func FormatHelp(c *cobra.Command) {
	fmt.Printf("%s\n\n", c.Long)
	c.Usage()
}

func FormatUsage(c *cobra.Command, usageFunc func(c *cobra.Command) error, exampleText string) {
	rescueStdout := os.Stdout
	rOut, wOut, _ := os.Pipe()
	c.SetOut(wOut)

	usageFunc(c)
	wOut.Close()

	var out, _ = ioutil.ReadAll(rOut)
	c.SetOut(rescueStdout)

	fmt.Println(colorUsage(c, string(out), exampleText))
}

func colorUsage(c *cobra.Command, usage string, exampleText string) string {
	subtext := lipgloss.NewStyle().Foreground(lipgloss.Color("242"))
	defaultText := lipgloss.NewStyle().Foreground(lipgloss.Color("246"))
	title := lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("4"))

	outStr := usage
	outStr = addColor(outStr, "Usage:", title)
	outStr = addColor(outStr, "Available Commands:", title)
	outStr = addColor(outStr, "Global Flags:", title)
	outStr = addColor(outStr, "Flags:", title)
	outStr = addColor(outStr, "[flags]", subtext)
	outStr = addColor(outStr, "[command]", subtext)
	if len(exampleText) > 0 {
		outStr = addColor(outStr, exampleText, defaultText)
	}

	c.Flags().VisitAll(func(flag *pflag.Flag) {
		outStr = addColor(outStr, flag.Usage, subtext)
		outStr = addColor(outStr, flag.Value.Type(), defaultText)
	})

	for _, c := range c.Commands() {
		outStr = addColor(outStr, c.Short, subtext)
	}

	return outStr
}

func PrettyPrintList(list []string) string {
	var formatted = []string{}
	numberStyle := lipgloss.NewStyle().Foreground(lipgloss.Color("242"))
	for i := 0; i < len(list); i++ {
		formatted = append(formatted, fmt.Sprintf("%s %s", numberStyle.Render(strconv.Itoa(i+1)+"."), list[i]))
	}
	return strings.Join(formatted, "\n")
}
