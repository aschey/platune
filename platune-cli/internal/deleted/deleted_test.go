package deleted

import (
	"io"
	"testing"

	"github.com/MarvinJWendt/testza"
	"github.com/aschey/platune/cli/v2/internal"
	"github.com/aschey/platune/cli/v2/test"
	platune "github.com/aschey/platune/client"
	"github.com/golang/mock/gomock"
	"google.golang.org/protobuf/types/known/emptypb"
)

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
