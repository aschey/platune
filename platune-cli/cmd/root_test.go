package cmd

import (
	"fmt"
	"io/ioutil"
	"os"
	"testing"

	"github.com/aschey/platune/cli/v2/test"
	"github.com/aschey/platune/cli/v2/utils"
	"github.com/golang/mock/gomock"
)

func Test(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	mock := test.NewMockPlayerClient(ctrl)
	mock.EXPECT().Pause(gomock.Any(), gomock.Any())
	utils.Client = utils.NewTestClient(mock)
	os.Args = append(os.Args, "pause")
	//rescueStdout := os.Stdout
	rOut, wOut, _ := os.Pipe()
	rootCmd.SetOut(wOut)

	if err := rootCmd.Execute(); err != nil {
		t.Errorf(err.Error())
	}
	wOut.Close()
	var out, _ = ioutil.ReadAll(rOut)
	fmt.Println(string(out))
}
