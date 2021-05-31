package test

import gomock "github.com/golang/mock/gomock"

func NewMatcher(customMatcher func(arg interface{}) bool) gomock.Matcher {
	return matcherCustomizer{customMatcher}
}

type matcherCustomizer struct {
	matcherFunction func(arg interface{}) bool
}

func (o matcherCustomizer) Matches(x interface{}) bool {
	return o.matcherFunction(x)
}

func (o matcherCustomizer) String() string {
	return "[call back function matcher has returned false]"
}
