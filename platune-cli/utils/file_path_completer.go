package utils

import (
	"fmt"
	"io/ioutil"
	"os"
	"os/user"
	"path/filepath"
	"strings"

	"github.com/c-bata/go-prompt"
)

type FilePathCompleter struct {
	Filter        func(fi os.FileInfo) bool
	IgnoreCase    bool
	fileListCache map[string][]prompt.Suggest
}

func CleanFilePath(path string) (dir, base string, err error) {
	if path == "" {
		return ".", "", nil
	}

	var endsWithSeparator bool
	if len(path) >= 1 && path[len(path)-1] == os.PathSeparator {
		endsWithSeparator = true
	}

	if len(path) >= 2 && path[0:2] == "~"+string(os.PathSeparator) {
		me, err := user.Current()
		if err != nil {
			return "", "", err
		}
		path = filepath.Join(me.HomeDir, path[1:])
	}
	path = filepath.Clean(os.ExpandEnv(path))
	dir = filepath.Dir(path)
	base = filepath.Base(path)

	if endsWithSeparator {
		dir = path + string(os.PathSeparator) // Append slash(in POSIX) if path ends with slash.
		base = ""                             // Set empty string if path ends with separator.
	}
	return dir, base, nil
}

func (c *FilePathCompleter) adjustCompletions(completions []prompt.Suggest, sub string) []prompt.Suggest {
	tokens := strings.Split(sub, " ")
	filteredCompletions := prompt.FilterHasPrefix(completions, sub, c.IgnoreCase)
	if len(tokens) > 1 {

		allExceptLast := strings.Join(tokens[0:len(tokens)-1], " ")
		newCompletions := []prompt.Suggest{}
		for _, completion := range filteredCompletions {
			completion.Text = completion.Text[len(allExceptLast)+1:]
			newCompletions = append(newCompletions, completion)
		}

		return newCompletions
	}
	return filteredCompletions
}

// Complete returns suggestions from your local file system.
func (c *FilePathCompleter) Complete(d prompt.Document, skipFirst bool) []prompt.Suggest {
	if c.fileListCache == nil {
		c.fileListCache = make(map[string][]prompt.Suggest, 4)
	}

	path := d.TextBeforeCursor()
	if skipFirst {
		path = strings.SplitN(path, " ", 2)[1]
	}
	dir, base, err := CleanFilePath(path)
	if err != nil {
		fmt.Println("completer: cannot get current user:" + err.Error())
		return nil
	}

	if cached, ok := c.fileListCache[dir]; ok {
		return c.adjustCompletions(cached, base)
	}

	files, err := ioutil.ReadDir(dir)
	if err != nil && os.IsNotExist(err) {
		return nil
	} else if err != nil {
		fmt.Println("completer: cannot read directory items:" + err.Error())
		return nil
	}

	suggests := make([]prompt.Suggest, 0, len(files))
	for _, f := range files {
		if c.Filter != nil && !c.Filter(f) {
			continue
		}
		suggests = append(suggests, prompt.Suggest{Text: f.Name()})
	}
	c.fileListCache[dir] = suggests
	return c.adjustCompletions(suggests, base)
}
