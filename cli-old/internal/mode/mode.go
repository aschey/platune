package mode

type Mode struct {
	modeList []ModeDef
}

func NewMode(first ModeDef) *Mode {
	return &Mode{modeList: []ModeDef{first}}
}

func NewDefaultMode() *Mode {
	return NewMode(NormalMode)
}

func (mode *Mode) Current() ModeDef {
	return mode.modeList[len(mode.modeList)-1]
}

func (mode *Mode) First() ModeDef {
	return mode.modeList[0]
}

func (mode *Mode) Set(nextMode ModeDef) {
	mode.modeList = append(mode.modeList, nextMode)
}

func (mode *Mode) Reset() {
	mode.modeList = []ModeDef{mode.First()}
}
