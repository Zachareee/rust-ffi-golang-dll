package main

// #include <stdlib.h>
import "C"
import (
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

//export GetStruct
func GetStruct() (count int32, strings **C.char) {
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

//export FreeStruct
func FreeStruct() {
	pinner.Unpin()
}

func main() {}
