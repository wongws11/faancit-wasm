package jyutping

import (
	"fmt"
	"testing"
)

func TestGetJyutping(t *testing.T) {
	tests := []struct {
		char   string
		wantOk bool
	}{
		{"德", true},
		{"紅", true},
		{"不存在的字zz", false},
	}

	for _, tt := range tests {
		got, ok := GetJyutping(tt.char)
		if ok != tt.wantOk {
			t.Errorf("GetJyutping(%q) ok = %v, want %v", tt.char, ok, tt.wantOk)
		}
		if ok && got.Sing == "" && got.Wan == "" {
			t.Errorf("GetJyutping(%q) returned empty JyutpingChar", tt.char)
		}
	}
}

func TestJyutpingCharString(t *testing.T) {
	j := JyutpingChar{Sing: "d", Wan: "ak", Tone: 1}
	got := j.String()
	want := "d ak 1"
	if got != want {
		t.Errorf("JyutpingChar.String() = %q, want %q", got, want)
	}
}

func TestIsStop(t *testing.T) {
	stops := []string{"b", "p", "d", "t", "g", "k", "gw", "kw", "z", "c"}
	nonStops := []string{"f", "s", "h", "m", "n", "ng", "l", "j", "w", ""}

	for _, s := range stops {
		if !isStop(JyutpingChar{Sing: s}) {
			t.Errorf("isStop(%q) = false, want true", s)
		}
	}
	for _, s := range nonStops {
		if isStop(JyutpingChar{Sing: s}) {
			t.Errorf("isStop(%q) = true, want false", s)
		}
	}
}

func TestIsFricative(t *testing.T) {
	fricatives := []string{"f", "s", "h"}
	nonFricatives := []string{"b", "d", "g", "m", "n", "l", ""}

	for _, s := range fricatives {
		if !isFricative(JyutpingChar{Sing: s}) {
			t.Errorf("isFricative(%q) = false, want true", s)
		}
	}
	for _, s := range nonFricatives {
		if isFricative(JyutpingChar{Sing: s}) {
			t.Errorf("isFricative(%q) = true, want false", s)
		}
	}
}

func TestIsPing(t *testing.T) {
	for tone := 1; tone <= 6; tone++ {
		got := isPing(JyutpingChar{Tone: tone})
		want := tone == 1 || tone == 4
		if got != want {
			t.Errorf("isPing(tone=%d) = %v, want %v", tone, got, want)
		}
	}
}

func TestIsYam(t *testing.T) {
	for tone := 1; tone <= 6; tone++ {
		got := isYam(JyutpingChar{Tone: tone})
		want := tone <= 3
		if got != want {
			t.Errorf("isYam(tone=%d) = %v, want %v", tone, got, want)
		}
	}
}

func TestGetFaancit_BasicCombination(t *testing.T) {
	// Basic: initial from upper, final+tone from lower
	upper := JyutpingChar{Sing: "d", Wan: "ak", Tone: 1}
	lower := JyutpingChar{Sing: "g", Wan: "ung", Tone: 4}

	result, _, _ := GetFaancit(upper, lower)

	if result.Sing != "d" {
		t.Errorf("GetFaancit initial = %q, want %q", result.Sing, "d")
	}
	if result.Wan != "ung" {
		t.Errorf("GetFaancit final = %q, want %q", result.Wan, "ung")
	}
}

func TestGetFaancit_AspirationRule(t *testing.T) {
	// 上字陽聲塞音 + 下字平聲 => aspiration
	upper := JyutpingChar{Sing: "b", Wan: "ei", Tone: 4} // yang, stop
	lower := JyutpingChar{Sing: "g", Wan: "ung", Tone: 4} // ping (tone 4)

	result, _, log := GetFaancit(upper, lower)

	if result.Sing != "p" {
		t.Errorf("aspiration rule: got initial %q, want %q", result.Sing, "p")
	}
	if log == "" {
		t.Error("expected log output for aspiration rule")
	}
}

func TestGetFaancit_LabiodentalShift(t *testing.T) {
	// 古無輕唇音: f -> b
	upper := JyutpingChar{Sing: "f", Wan: "ong", Tone: 1}
	lower := JyutpingChar{Sing: "g", Wan: "ung", Tone: 1}

	result, _, log := GetFaancit(upper, lower)

	if result.Sing != "b" {
		t.Errorf("labiodental shift: got initial %q, want %q", result.Sing, "b")
	}
	if log == "" {
		t.Error("expected log output for labiodental shift")
	}
}

func TestGetFaancit_YinYangToneMapping(t *testing.T) {
	// 上字陰下字陽: tone 4->1, 5->2, 6->3
	tests := []struct {
		lowerTone int
		wantTone  int
	}{
		{4, 1},
		{5, 2},
		{6, 3},
	}

	for _, tt := range tests {
		t.Run(fmt.Sprintf("lower_tone_%d", tt.lowerTone), func(t *testing.T) {
			upper := JyutpingChar{Sing: "l", Wan: "ai", Tone: 1}             // yin, non-stop
			lower := JyutpingChar{Sing: "l", Wan: "ung", Tone: tt.lowerTone} // yang

			result, _, _ := GetFaancit(upper, lower)

			if result.Tone != tt.wantTone {
				t.Errorf("yin-yang mapping: lower tone %d => got %d, want %d", tt.lowerTone, result.Tone, tt.wantTone)
			}
		})
	}
}

func TestGetFaancit_YangYinToneMapping(t *testing.T) {
	// 上字陽下字陰: tone 1->4, 2->5, 3->6
	tests := []struct {
		lowerTone int
		wantTone  int
	}{
		{1, 4},
		{2, 5},
		{3, 6},
	}

	for _, tt := range tests {
		t.Run(fmt.Sprintf("lower_tone_%d", tt.lowerTone), func(t *testing.T) {
			upper := JyutpingChar{Sing: "l", Wan: "ai", Tone: 4}             // yang, non-stop
			lower := JyutpingChar{Sing: "l", Wan: "ung", Tone: tt.lowerTone} // yin

			result, _, _ := GetFaancit(upper, lower)

			if result.Tone != tt.wantTone {
				t.Errorf("yang-yin mapping: lower tone %d => got %d, want %d", tt.lowerTone, result.Tone, tt.wantTone)
			}
		})
	}
}
