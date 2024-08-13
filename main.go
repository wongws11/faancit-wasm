//go:build js && wasm

package main

import (
	"fmt"
	"syscall/js"

	"github.com/wongws11/faancit-wasm/jyutping"
)

func inputOnChange(this js.Value, args []js.Value) interface{} {
	// Get the value of the input element
	value := this.Get("value").String()

	if len([]rune(value)) != 2 {
		return nil
	}

	jyutpingUpper, okUpper := jyutping.GetJyutping(string([]rune(value)[0]))
	jyutpingLower, okLower := jyutping.GetJyutping(string([]rune(value)[1]))

	if !okUpper || !okLower {
		return nil
	}

	result, pronunciations, log := jyutping.GetFaancit(jyutpingUpper, jyutpingLower)

	updateOutput(result.String(), pronunciations)

	if log != "" {
		fmt.Println(log)
	}

	return nil
}

func updateOutput(value string, pronunciations []string) {
	js.Global().Get("document").Call("getElementById", "output").Set("innerText", value)
	js.Global().Get("document").Call("getElementById", "pronunciations").Set("innerText", fmt.Sprintf("%v", pronunciations))
}

func main() {
	inputOnChangeCallback := js.FuncOf(inputOnChange)
	defer inputOnChangeCallback.Release()

	input := js.Global().Get("document").Call("getElementById", "input")

	// On change of the input element, call the callback function
	input.Call("addEventListener", "input", inputOnChangeCallback)

	// Prevent the Go program from exiting immediately
	select {}
}
