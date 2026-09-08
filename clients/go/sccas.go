// Package sccas provides pure Go zero-CGO bindings for SC-Substrate WebAssembly Substrate.
package sccas

import (
	"bytes"
	"context"
	"encoding/binary"
	"errors"
	"fmt"
)

var (
	PreambleCanary     = []byte("SCCASWAS")
	TrailingSealCanary = []byte("SCS_SEAL")
)

type OpCode uint32

const (
	OpEvalSymbolic         OpCode = 1
	OpIntegrateMonodromy   OpCode = 2
	OpDixonSolve           OpCode = 3
	OpDeepPolyVerify       OpCode = 4
	OpArithmeticSimplify   OpCode = 5
	OpCliffordMultivector  OpCode = 6
	OpTensorSlice          OpCode = 7
)

type EvaluationResult struct {
	StatusCode        uint32
	Payload           string
	AttestationDigest [32]byte
}

type Client struct {
	allocFn    func(ctx context.Context, size uint32) (uint32, error)
	freeFn     func(ctx context.Context, offset, size uint32) error
	dispatchFn func(ctx context.Context, inOff, inLen, outOff, outCap uint32) (uint32, error)
	readMem    func(offset, length uint32) ([]byte, error)
	writeMem   func(offset uint32, data []byte) error
}

func (c *Client) Execute(ctx context.Context, op OpCode, flags uint32, payload string) (*EvaluationResult, error) {
	payloadBytes := []byte(payload)
	req := bytes.NewBuffer(make([]byte, 0, 20+len(payloadBytes)))
	req.Write(PreambleCanary)
	_ = binary.Write(req, binary.LittleEndian, uint32(op))
	_ = binary.Write(req, binary.LittleEndian, flags)
	_ = binary.Write(req, binary.LittleEndian, uint32(len(payloadBytes)))
	req.Write(payloadBytes)

	inOff, err := c.allocFn(ctx, uint32(req.Len()))
	if err != nil || inOff == 0 {
		return nil, errors.New("failed to allocate linear memory for request")
	}
	defer c.freeFn(ctx, inOff, uint32(req.Len()))

	if err := c.writeMem(inOff, req.Bytes()); err != nil {
		return nil, err
	}

	outCap := uint32(65536)
	outOff, err := c.allocFn(ctx, outCap)
	if err != nil || outOff == 0 {
		return nil, errors.New("failed to allocate linear memory for response")
	}
	defer c.freeFn(ctx, outOff, outCap)

	written, err := c.dispatchFn(ctx, inOff, uint32(req.Len()), outOff, outCap)
	if err != nil || written == 0 {
		return nil, fmt.Errorf("wasm dispatch error: %w", err)
	}

	respBytes, err := c.readMem(outOff, written)
	if err != nil {
		return nil, err
	}

	if len(respBytes) < 56 || !bytes.Equal(respBytes[:8], PreambleCanary) {
		return nil, errors.New("corrupt response envelope")
	}

	status := binary.LittleEndian.Uint32(respBytes[8:12])
	pLen := binary.LittleEndian.Uint32(respBytes[12:16])
	resText := string(respBytes[16 : 16+pLen])

	var digest [32]byte
	copy(digest[:], respBytes[16+pLen:16+pLen+32])

	return &EvaluationResult{
		StatusCode:        status,
		Payload:           resText,
		AttestationDigest: digest,
	}, nil
}
