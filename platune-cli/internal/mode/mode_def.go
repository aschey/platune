package mode

type ModeDef string

const (
	NormalMode   ModeDef = ">>> "
	SetQueueMode ModeDef = "set-queue> "
	AlbumMode    ModeDef = "album> "
	SongMode     ModeDef = "song> "
)
