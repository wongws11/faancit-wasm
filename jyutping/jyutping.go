package jyutping

import (
	_ "embed"
	"encoding/json"
	"fmt"
	"log"
	"strings"
	"time"
)

type JyutpingChar struct {
	Sing string `json:"sing"`
	Wan  string `json:"wan"`
	Tone int    `json:"tone"`
}

func (j JyutpingChar) String() string {
	return fmt.Sprintf("%s %s %d", j.Sing, j.Wan, j.Tone)
}

//go:embed jyutping.json
var jsonData []byte

var jyutpingMap map[string]JyutpingChar = loadMap()
var jyutpingReverseMap map[JyutpingChar][]string = loadReverseMap()

func loadMap() map[string]JyutpingChar {
	startTime := time.Now()
	var result map[string]JyutpingChar

	if err := json.Unmarshal(jsonData, &result); err != nil {
		log.Fatalf("Failed to parse JSON: %s", err)
	}

	fmt.Println("loadMap took", time.Since(startTime))

	return result
}

func loadReverseMap() map[JyutpingChar][]string {
	startTime := time.Now()
	result := make(map[JyutpingChar][]string)

	for k, v := range jyutpingMap {
		if val, ok := result[v]; ok {
			result[v] = append(val, k)
		} else {
			result[v] = []string{k}
		}
	}

	fmt.Println("loadReverseMap took", time.Since(startTime))

	return result
}

func GetJyutping(char string) (JyutpingChar, bool) {
	jyutping, ok := jyutpingMap[char]
	return jyutping, ok
}

// 塞音
func isStop(char JyutpingChar) bool {
	switch char.Sing {
	case "b", "p", "d", "t", "g", "k", "gw", "kw", "z", "c":
		return true
	default:
		return false
	}
}

// 擦音
func isFricative(char JyutpingChar) bool {
	switch char.Sing {
	case "f", "s", "h":
		return true
	default:
		return false
	}
}

// 平聲
func isPing(char JyutpingChar) bool {
	return char.Tone == 1 || char.Tone == 4
}

// 陰陽
func isYam(char JyutpingChar) bool {
	return char.Tone <= 3
}

var aspiratedMap = map[string]string{
	"b":  "p",
	"d":  "t",
	"g":  "k",
	"gw": "kw",
	"z":  "c",
}

var tenuisMap = map[string]string{
	"p":  "b",
	"t":  "d",
	"k":  "g",
	"kw": "gw",
	"c":  "z",
}

func GetFaancit(charUpper JyutpingChar, charLower JyutpingChar) (JyutpingChar, []string, string) {
	result := JyutpingChar{
		Sing: charUpper.Sing,
		Wan:  charLower.Wan,
		Tone: charLower.Tone,
	}

	var builder strings.Builder

	if !isYam(charUpper) && isStop(charUpper) {
		// 上字陽聲塞音
		builder.WriteString("上字陽聲塞音\n")
		if isPing(charLower) {
			// 下字平聲則聲母送氣
			builder.WriteString("下字平聲則聲母送氣\n")
			if aspirated, ok := aspiratedMap[charUpper.Sing]; ok {
				result.Sing = aspirated
			}
		} else {
			// 下字仄聲則聲母不送氣
			builder.WriteString("下字仄聲則聲母不送氣\n")
			if tenuis, ok := tenuisMap[charUpper.Sing]; ok {
				result.Sing = tenuis
			}
		}
	}

	if charUpper.Sing == "f" {
		// 古無輕唇音
		builder.WriteString("古無輕唇音\n")
		result.Sing = "b"
	}

	// fmt.Println("Upper:", isYam(charUpper), "Lower:", isYam(charLower))

	if isYam(charUpper) && !isYam(charLower) {
		// 上字陰下字陽
		builder.WriteString("上字陰下字陽\n")
		switch charLower.Tone {
		case 4:
			result.Tone = 1
		case 5:
			result.Tone = 2
		case 6:
			result.Tone = 3
		}
	} else if !isYam(charUpper) && isYam(charLower) {
		// 上字陽下字陰
		builder.WriteString("上字陽下字陰\n")
		switch charLower.Tone {
		case 1:
			result.Tone = 4
		case 2:
			result.Tone = 5
		case 3:
			result.Tone = 6
		}
	}

	if !isYam(charUpper) && (isStop(charUpper) || isFricative(charLower)) {
		// 上字陽聲而上字塞音或下字擦音
		builder.WriteString("上字陽聲而上字塞音或下字擦音\n")
		switch result.Tone {
		case 2:
			result.Tone = 3
		case 5:
			result.Tone = 6
		}
	}

	samePronunciations := jyutpingReverseMap[result]

	return result, samePronunciations, builder.String()
}
