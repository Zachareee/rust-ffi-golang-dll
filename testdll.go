package main

// #include <stdlib.h>
import "C"
import (
	"fmt"
	"runtime"
	"time"
	"unsafe"
)

var pinner runtime.Pinner

//export GetString
func GetString() *C.char {
	return C.CString(time.Now().Local().String())
}

//export FreeString
func FreeString(str *C.char) {
	C.free(unsafe.Pointer(str))
}

//export GetArray
func GetArray() (count int32, strings **C.char) {
	countries := []string{"Germany", "Korea", "Japan"}
	count = int32(len(countries))
	s := make([]*C.char, 0, count)

	for _, str := range countries {
		s = append(s, C.CString(str))
	}

	strings = unsafe.SliceData(s)
	pinner.Pin(strings)
	return
}

//export Uninit
func Uninit() {
	pinner.Unpin()
}

//export PrintString
func PrintString(ptr *C.char) {
	fmt.Println(C.GoString(ptr))
}

func main() {}
