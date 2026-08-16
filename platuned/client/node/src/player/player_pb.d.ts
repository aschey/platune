// package: platune.player.v1
// file: player.proto

/* tslint:disable */
/* eslint-disable */

import * as jspb from "google-protobuf";
import * as google_protobuf_duration_pb from "google-protobuf/google/protobuf/duration_pb";
import * as google_protobuf_empty_pb from "google-protobuf/google/protobuf/empty_pb";

export class Track extends jspb.Message { 
    getUrl(): string;
    setUrl(value: string): Track;

    hasMetadata(): boolean;
    clearMetadata(): void;
    getMetadata(): Metadata | undefined;
    setMetadata(value?: Metadata): Track;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Track.AsObject;
    static toObject(includeInstance: boolean, msg: Track): Track.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Track, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Track;
    static deserializeBinaryFromReader(message: Track, reader: jspb.BinaryReader): Track;
}

export namespace Track {
    export type AsObject = {
        url: string,
        metadata?: Metadata.AsObject,
    }
}

export class QueueRequest extends jspb.Message { 
    clearQueueList(): void;
    getQueueList(): Array<Track>;
    setQueueList(value: Array<Track>): QueueRequest;
    addQueue(value?: Track, index?: number): Track;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): QueueRequest.AsObject;
    static toObject(includeInstance: boolean, msg: QueueRequest): QueueRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: QueueRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): QueueRequest;
    static deserializeBinaryFromReader(message: QueueRequest, reader: jspb.BinaryReader): QueueRequest;
}

export namespace QueueRequest {
    export type AsObject = {
        queueList: Array<Track.AsObject>,
    }
}

export class SeekRequest extends jspb.Message { 

    hasTime(): boolean;
    clearTime(): void;
    getTime(): google_protobuf_duration_pb.Duration | undefined;
    setTime(value?: google_protobuf_duration_pb.Duration): SeekRequest;
    getMode(): SeekMode;
    setMode(value: SeekMode): SeekRequest;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): SeekRequest.AsObject;
    static toObject(includeInstance: boolean, msg: SeekRequest): SeekRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: SeekRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): SeekRequest;
    static deserializeBinaryFromReader(message: SeekRequest, reader: jspb.BinaryReader): SeekRequest;
}

export namespace SeekRequest {
    export type AsObject = {
        time?: google_protobuf_duration_pb.Duration.AsObject,
        mode: SeekMode,
    }
}

export class SetVolumeRequest extends jspb.Message { 
    getVolume(): number;
    setVolume(value: number): SetVolumeRequest;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): SetVolumeRequest.AsObject;
    static toObject(includeInstance: boolean, msg: SetVolumeRequest): SetVolumeRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: SetVolumeRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): SetVolumeRequest;
    static deserializeBinaryFromReader(message: SetVolumeRequest, reader: jspb.BinaryReader): SetVolumeRequest;
}

export namespace SetVolumeRequest {
    export type AsObject = {
        volume: number,
    }
}

export class EventResponse extends jspb.Message { 
    getEvent(): Event;
    setEvent(value: Event): EventResponse;

    hasState(): boolean;
    clearState(): void;
    getState(): State | undefined;
    setState(value?: State): EventResponse;

    hasSeekData(): boolean;
    clearSeekData(): void;
    getSeekData(): SeekResponse | undefined;
    setSeekData(value?: SeekResponse): EventResponse;

    hasProgress(): boolean;
    clearProgress(): void;
    getProgress(): PositionResponse | undefined;
    setProgress(value?: PositionResponse): EventResponse;

    getEventPayloadCase(): EventResponse.EventPayloadCase;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): EventResponse.AsObject;
    static toObject(includeInstance: boolean, msg: EventResponse): EventResponse.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: EventResponse, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): EventResponse;
    static deserializeBinaryFromReader(message: EventResponse, reader: jspb.BinaryReader): EventResponse;
}

export namespace EventResponse {
    export type AsObject = {
        event: Event,
        state?: State.AsObject,
        seekData?: SeekResponse.AsObject,
        progress?: PositionResponse.AsObject,
    }

    export enum EventPayloadCase {
        EVENT_PAYLOAD_NOT_SET = 0,
        STATE = 2,
        SEEK_DATA = 3,
        PROGRESS = 4,
    }

}

export class Metadata extends jspb.Message { 

    hasArtist(): boolean;
    clearArtist(): void;
    getArtist(): string | undefined;
    setArtist(value: string): Metadata;

    hasAlbumArtist(): boolean;
    clearAlbumArtist(): void;
    getAlbumArtist(): string | undefined;
    setAlbumArtist(value: string): Metadata;

    hasAlbum(): boolean;
    clearAlbum(): void;
    getAlbum(): string | undefined;
    setAlbum(value: string): Metadata;

    hasSong(): boolean;
    clearSong(): void;
    getSong(): string | undefined;
    setSong(value: string): Metadata;

    hasTrackNumber(): boolean;
    clearTrackNumber(): void;
    getTrackNumber(): number | undefined;
    setTrackNumber(value: number): Metadata;

    hasDuration(): boolean;
    clearDuration(): void;
    getDuration(): google_protobuf_duration_pb.Duration | undefined;
    setDuration(value?: google_protobuf_duration_pb.Duration): Metadata;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Metadata.AsObject;
    static toObject(includeInstance: boolean, msg: Metadata): Metadata.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Metadata, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Metadata;
    static deserializeBinaryFromReader(message: Metadata, reader: jspb.BinaryReader): Metadata;
}

export namespace Metadata {
    export type AsObject = {
        artist?: string,
        albumArtist?: string,
        album?: string,
        song?: string,
        trackNumber?: number,
        duration?: google_protobuf_duration_pb.Duration.AsObject,
    }
}

export class State extends jspb.Message { 
    clearQueueList(): void;
    getQueueList(): Array<string>;
    setQueueList(value: Array<string>): State;
    addQueue(value: string, index?: number): string;
    getQueuePosition(): number;
    setQueuePosition(value: number): State;
    getStatus(): PlayerStatus;
    setStatus(value: PlayerStatus): State;
    getVolume(): number;
    setVolume(value: number): State;

    hasMetadata(): boolean;
    clearMetadata(): void;
    getMetadata(): Metadata | undefined;
    setMetadata(value?: Metadata): State;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): State.AsObject;
    static toObject(includeInstance: boolean, msg: State): State.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: State, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): State;
    static deserializeBinaryFromReader(message: State, reader: jspb.BinaryReader): State;
}

export namespace State {
    export type AsObject = {
        queueList: Array<string>,
        queuePosition: number,
        status: PlayerStatus,
        volume: number,
        metadata?: Metadata.AsObject,
    }
}

export class PositionResponse extends jspb.Message { 

    hasPosition(): boolean;
    clearPosition(): void;
    getPosition(): google_protobuf_duration_pb.Duration | undefined;
    setPosition(value?: google_protobuf_duration_pb.Duration): PositionResponse;

    hasRetrievalTime(): boolean;
    clearRetrievalTime(): void;
    getRetrievalTime(): google_protobuf_duration_pb.Duration | undefined;
    setRetrievalTime(value?: google_protobuf_duration_pb.Duration): PositionResponse;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): PositionResponse.AsObject;
    static toObject(includeInstance: boolean, msg: PositionResponse): PositionResponse.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: PositionResponse, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): PositionResponse;
    static deserializeBinaryFromReader(message: PositionResponse, reader: jspb.BinaryReader): PositionResponse;
}

export namespace PositionResponse {
    export type AsObject = {
        position?: google_protobuf_duration_pb.Duration.AsObject,
        retrievalTime?: google_protobuf_duration_pb.Duration.AsObject,
    }
}

export class SeekResponse extends jspb.Message { 

    hasState(): boolean;
    clearState(): void;
    getState(): State | undefined;
    setState(value?: State): SeekResponse;
    getSeekMillis(): number;
    setSeekMillis(value: number): SeekResponse;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): SeekResponse.AsObject;
    static toObject(includeInstance: boolean, msg: SeekResponse): SeekResponse.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: SeekResponse, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): SeekResponse;
    static deserializeBinaryFromReader(message: SeekResponse, reader: jspb.BinaryReader): SeekResponse;
}

export namespace SeekResponse {
    export type AsObject = {
        state?: State.AsObject,
        seekMillis: number,
    }
}

export class StatusResponse extends jspb.Message { 

    hasState(): boolean;
    clearState(): void;
    getState(): State | undefined;
    setState(value?: State): StatusResponse;

    hasProgress(): boolean;
    clearProgress(): void;
    getProgress(): PositionResponse | undefined;
    setProgress(value?: PositionResponse): StatusResponse;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): StatusResponse.AsObject;
    static toObject(includeInstance: boolean, msg: StatusResponse): StatusResponse.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: StatusResponse, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): StatusResponse;
    static deserializeBinaryFromReader(message: StatusResponse, reader: jspb.BinaryReader): StatusResponse;
}

export namespace StatusResponse {
    export type AsObject = {
        state?: State.AsObject,
        progress?: PositionResponse.AsObject,
    }
}

export class DevicesResponse extends jspb.Message { 
    clearDevicesList(): void;
    getDevicesList(): Array<string>;
    setDevicesList(value: Array<string>): DevicesResponse;
    addDevices(value: string, index?: number): string;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): DevicesResponse.AsObject;
    static toObject(includeInstance: boolean, msg: DevicesResponse): DevicesResponse.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: DevicesResponse, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): DevicesResponse;
    static deserializeBinaryFromReader(message: DevicesResponse, reader: jspb.BinaryReader): DevicesResponse;
}

export namespace DevicesResponse {
    export type AsObject = {
        devicesList: Array<string>,
    }
}

export class SetOutputDeviceRequest extends jspb.Message { 

    hasDevice(): boolean;
    clearDevice(): void;
    getDevice(): string | undefined;
    setDevice(value: string): SetOutputDeviceRequest;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): SetOutputDeviceRequest.AsObject;
    static toObject(includeInstance: boolean, msg: SetOutputDeviceRequest): SetOutputDeviceRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: SetOutputDeviceRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): SetOutputDeviceRequest;
    static deserializeBinaryFromReader(message: SetOutputDeviceRequest, reader: jspb.BinaryReader): SetOutputDeviceRequest;
}

export namespace SetOutputDeviceRequest {
    export type AsObject = {
        device?: string,
    }
}

export enum Event {
    START_QUEUE = 0,
    QUEUE_UPDATED = 1,
    STOP = 2,
    PAUSE = 3,
    RESUME = 4,
    TRACK_CHANGED = 5,
    SET_VOLUME = 6,
    SEEK = 7,
    QUEUE_ENDED = 8,
    POSITION = 9,
}

export enum PlayerStatus {
    PLAYING = 0,
    STOPPED = 1,
    PAUSED = 2,
}

export enum SeekMode {
    FORWARD = 0,
    BACKWARD = 1,
    ABSOLUTE = 2,
}
