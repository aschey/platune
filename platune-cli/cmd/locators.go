package cmd

import (
	"context"

	"github.com/aschey/platune/cli/v2/internal"
	"github.com/aschey/platune/cli/v2/internal/deleted"
	"github.com/aschey/platune/cli/v2/internal/search"
	"github.com/spf13/cobra"
	"go.uber.org/zap"
)

type Dependency int

const (
	loggerKey Dependency = iota
	clientKey
	stateKey
	deletedKey
	searchKey
)

func RegisterLogger(ctx context.Context, logger *zap.Logger) context.Context {
	return context.WithValue(ctx, loggerKey, logger)
}

func GetClient(cmd *cobra.Command) *internal.PlatuneClient {
	ctx := cmd.Context()
	return ctx.Value(clientKey).(*internal.PlatuneClient)
}

func RegisterClient(ctx context.Context, client *internal.PlatuneClient) context.Context {
	return context.WithValue(ctx, clientKey, client)
}

func GetState(cmd *cobra.Command) *cmdState {
	ctx := cmd.Context()
	return ctx.Value(stateKey).(*cmdState)
}

func RegisterState(ctx context.Context, state *cmdState) context.Context {
	return context.WithValue(ctx, stateKey, state)
}

func GetDeleted(cmd *cobra.Command) *deleted.Deleted {
	ctx := cmd.Context()
	return ctx.Value(deletedKey).(*deleted.Deleted)
}

func RegisterDeleted(ctx context.Context, deleted *deleted.Deleted) context.Context {
	return context.WithValue(ctx, deletedKey, deleted)
}

func GetSearch(cmd *cobra.Command) *search.Search {
	ctx := cmd.Context()
	return ctx.Value(searchKey).(*search.Search)
}

func RegisterSearch(ctx context.Context, search *search.Search) context.Context {
	return context.WithValue(ctx, searchKey, search)
}
