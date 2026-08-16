// package: platune.management.v1
// file: management.proto

/* tslint:disable */
/* eslint-disable */

import * as jspb from "google-protobuf";
import * as google_protobuf_duration_pb from "google-protobuf/google/protobuf/duration_pb";
import * as google_protobuf_empty_pb from "google-protobuf/google/protobuf/empty_pb";

export class Progress extends jspb.Message { 
    getJob(): string;
    setJob(value: string): Progress;
    getPercentage(): number;
    setPercentage(value: number): Progress;
    getFinished(): boolean;
    setFinished(value: boolean): Progress;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Progress.AsObject;
    static toObject(includeInstance: boolean, msg: Progress): Progress.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Progress, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Progress;
    static deserializeBinaryFromReader(message: Progress, reader: jspb.BinaryReader): Progress;
}

export namespace Progress {
    export type AsObject = {
        job: string,
        percentage: number,
        finished: boolean,
    }
}

export class FoldersMessage extends jspb.Message { 
    clearFoldersList(): void;
    getFoldersList(): Array<string>;
    setFoldersList(value: Array<string>): FoldersMessage;
    addFolders(value: string, index?: number): string;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): FoldersMessage.AsObject;
    static toObject(includeInstance: boolean, msg: FoldersMessage): FoldersMessage.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: FoldersMessage, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): FoldersMessage;
    static deserializeBinaryFromReader(message: FoldersMessage, reader: jspb.BinaryReader): FoldersMessage;
}

export namespace FoldersMessage {
    export type AsObject = {
        foldersList: Array<string>,
    }
}

export class RegisteredMountMessage extends jspb.Message { 
    getMount(): string;
    setMount(value: string): RegisteredMountMessage;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): RegisteredMountMessage.AsObject;
    static toObject(includeInstance: boolean, msg: RegisteredMountMessage): RegisteredMountMessage.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: RegisteredMountMessage, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): RegisteredMountMessage;
    static deserializeBinaryFromReader(message: RegisteredMountMessage, reader: jspb.BinaryReader): RegisteredMountMessage;
}

export namespace RegisteredMountMessage {
    export type AsObject = {
        mount: string,
    }
}

export class IdMessage extends jspb.Message { 
    clearIdsList(): void;
    getIdsList(): Array<number>;
    setIdsList(value: Array<number>): IdMessage;
    addIds(value: number, index?: number): number;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): IdMessage.AsObject;
    static toObject(includeInstance: boolean, msg: IdMessage): IdMessage.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: IdMessage, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): IdMessage;
    static deserializeBinaryFromReader(message: IdMessage, reader: jspb.BinaryReader): IdMessage;
}

export namespace IdMessage {
    export type AsObject = {
        idsList: Array<number>,
    }
}

export class PathMessage extends jspb.Message { 
    getPath(): string;
    setPath(value: string): PathMessage;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): PathMessage.AsObject;
    static toObject(includeInstance: boolean, msg: PathMessage): PathMessage.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: PathMessage, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): PathMessage;
    static deserializeBinaryFromReader(message: PathMessage, reader: jspb.BinaryReader): PathMessage;
}

export namespace PathMessage {
    export type AsObject = {
        path: string,
    }
}

export class SearchRequest extends jspb.Message { 
    getQuery(): string;
    setQuery(value: string): SearchRequest;

    hasStartSeparator(): boolean;
    clearStartSeparator(): void;
    getStartSeparator(): string | undefined;
    setStartSeparator(value: string): SearchRequest;

    hasEndSeparator(): boolean;
    clearEndSeparator(): void;
    getEndSeparator(): string | undefined;
    setEndSeparator(value: string): SearchRequest;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): SearchRequest.AsObject;
    static toObject(includeInstance: boolean, msg: SearchRequest): SearchRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: SearchRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): SearchRequest;
    static deserializeBinaryFromReader(message: SearchRequest, reader: jspb.BinaryReader): SearchRequest;
}

export namespace SearchRequest {
    export type AsObject = {
        query: string,
        startSeparator?: string,
        endSeparator?: string,
    }
}

export class LookupRequest extends jspb.Message { 
    getEntryType(): EntryType;
    setEntryType(value: EntryType): LookupRequest;
    clearCorrelationIdsList(): void;
    getCorrelationIdsList(): Array<number>;
    setCorrelationIdsList(value: Array<number>): LookupRequest;
    addCorrelationIds(value: number, index?: number): number;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): LookupRequest.AsObject;
    static toObject(includeInstance: boolean, msg: LookupRequest): LookupRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: LookupRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): LookupRequest;
    static deserializeBinaryFromReader(message: LookupRequest, reader: jspb.BinaryReader): LookupRequest;
}

export namespace LookupRequest {
    export type AsObject = {
        entryType: EntryType,
        correlationIdsList: Array<number>,
    }
}

export class SongResponse extends jspb.Message { 

    hasSong(): boolean;
    clearSong(): void;
    getSong(): LookupEntry | undefined;
    setSong(value?: LookupEntry): SongResponse;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): SongResponse.AsObject;
    static toObject(includeInstance: boolean, msg: SongResponse): SongResponse.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: SongResponse, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): SongResponse;
    static deserializeBinaryFromReader(message: SongResponse, reader: jspb.BinaryReader): SongResponse;
}

export namespace SongResponse {
    export type AsObject = {
        song?: LookupEntry.AsObject,
    }
}

export class AlbumResponse extends jspb.Message { 
    clearEntriesList(): void;
    getEntriesList(): Array<AlbumEntry>;
    setEntriesList(value: Array<AlbumEntry>): AlbumResponse;
    addEntries(value?: AlbumEntry, index?: number): AlbumEntry;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): AlbumResponse.AsObject;
    static toObject(includeInstance: boolean, msg: AlbumResponse): AlbumResponse.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: AlbumResponse, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): AlbumResponse;
    static deserializeBinaryFromReader(message: AlbumResponse, reader: jspb.BinaryReader): AlbumResponse;
}

export namespace AlbumResponse {
    export type AsObject = {
        entriesList: Array<AlbumEntry.AsObject>,
    }
}

export class AlbumEntry extends jspb.Message { 
    getAlbum(): string;
    setAlbum(value: string): AlbumEntry;
    getAlbumId(): number;
    setAlbumId(value: number): AlbumEntry;
    getAlbumArtist(): string;
    setAlbumArtist(value: string): AlbumEntry;
    getAlbumArtistId(): number;
    setAlbumArtistId(value: number): AlbumEntry;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): AlbumEntry.AsObject;
    static toObject(includeInstance: boolean, msg: AlbumEntry): AlbumEntry.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: AlbumEntry, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): AlbumEntry;
    static deserializeBinaryFromReader(message: AlbumEntry, reader: jspb.BinaryReader): AlbumEntry;
}

export namespace AlbumEntry {
    export type AsObject = {
        album: string,
        albumId: number,
        albumArtist: string,
        albumArtistId: number,
    }
}

export class LookupEntry extends jspb.Message { 
    getArtist(): string;
    setArtist(value: string): LookupEntry;
    getAlbumArtist(): string;
    setAlbumArtist(value: string): LookupEntry;
    getAlbum(): string;
    setAlbum(value: string): LookupEntry;
    getSong(): string;
    setSong(value: string): LookupEntry;
    getPath(): string;
    setPath(value: string): LookupEntry;
    getTrackNumber(): number;
    setTrackNumber(value: number): LookupEntry;

    hasDuration(): boolean;
    clearDuration(): void;
    getDuration(): google_protobuf_duration_pb.Duration | undefined;
    setDuration(value?: google_protobuf_duration_pb.Duration): LookupEntry;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): LookupEntry.AsObject;
    static toObject(includeInstance: boolean, msg: LookupEntry): LookupEntry.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: LookupEntry, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): LookupEntry;
    static deserializeBinaryFromReader(message: LookupEntry, reader: jspb.BinaryReader): LookupEntry;
}

export namespace LookupEntry {
    export type AsObject = {
        artist: string,
        albumArtist: string,
        album: string,
        song: string,
        path: string,
        trackNumber: number,
        duration?: google_protobuf_duration_pb.Duration.AsObject,
    }
}

export class LookupResponse extends jspb.Message { 
    clearEntriesList(): void;
    getEntriesList(): Array<LookupEntry>;
    setEntriesList(value: Array<LookupEntry>): LookupResponse;
    addEntries(value?: LookupEntry, index?: number): LookupEntry;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): LookupResponse.AsObject;
    static toObject(includeInstance: boolean, msg: LookupResponse): LookupResponse.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: LookupResponse, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): LookupResponse;
    static deserializeBinaryFromReader(message: LookupResponse, reader: jspb.BinaryReader): LookupResponse;
}

export namespace LookupResponse {
    export type AsObject = {
        entriesList: Array<LookupEntry.AsObject>,
    }
}

export class SearchResult extends jspb.Message { 
    getEntry(): string;
    setEntry(value: string): SearchResult;
    getEntryType(): EntryType;
    setEntryType(value: EntryType): SearchResult;

    hasArtist(): boolean;
    clearArtist(): void;
    getArtist(): string | undefined;
    setArtist(value: string): SearchResult;
    clearCorrelationIdsList(): void;
    getCorrelationIdsList(): Array<number>;
    setCorrelationIdsList(value: Array<number>): SearchResult;
    addCorrelationIds(value: number, index?: number): number;
    getDescription(): string;
    setDescription(value: string): SearchResult;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): SearchResult.AsObject;
    static toObject(includeInstance: boolean, msg: SearchResult): SearchResult.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: SearchResult, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): SearchResult;
    static deserializeBinaryFromReader(message: SearchResult, reader: jspb.BinaryReader): SearchResult;
}

export namespace SearchResult {
    export type AsObject = {
        entry: string,
        entryType: EntryType,
        artist?: string,
        correlationIdsList: Array<number>,
        description: string,
    }
}

export class SearchResponse extends jspb.Message { 
    clearResultsList(): void;
    getResultsList(): Array<SearchResult>;
    setResultsList(value: Array<SearchResult>): SearchResponse;
    addResults(value?: SearchResult, index?: number): SearchResult;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): SearchResponse.AsObject;
    static toObject(includeInstance: boolean, msg: SearchResponse): SearchResponse.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: SearchResponse, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): SearchResponse;
    static deserializeBinaryFromReader(message: SearchResponse, reader: jspb.BinaryReader): SearchResponse;
}

export namespace SearchResponse {
    export type AsObject = {
        resultsList: Array<SearchResult.AsObject>,
    }
}

export class DeletedResult extends jspb.Message { 
    getPath(): string;
    setPath(value: string): DeletedResult;
    getId(): number;
    setId(value: number): DeletedResult;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): DeletedResult.AsObject;
    static toObject(includeInstance: boolean, msg: DeletedResult): DeletedResult.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: DeletedResult, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): DeletedResult;
    static deserializeBinaryFromReader(message: DeletedResult, reader: jspb.BinaryReader): DeletedResult;
}

export namespace DeletedResult {
    export type AsObject = {
        path: string,
        id: number,
    }
}

export class GetDeletedResponse extends jspb.Message { 
    clearResultsList(): void;
    getResultsList(): Array<DeletedResult>;
    setResultsList(value: Array<DeletedResult>): GetDeletedResponse;
    addResults(value?: DeletedResult, index?: number): DeletedResult;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): GetDeletedResponse.AsObject;
    static toObject(includeInstance: boolean, msg: GetDeletedResponse): GetDeletedResponse.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: GetDeletedResponse, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): GetDeletedResponse;
    static deserializeBinaryFromReader(message: GetDeletedResponse, reader: jspb.BinaryReader): GetDeletedResponse;
}

export namespace GetDeletedResponse {
    export type AsObject = {
        resultsList: Array<DeletedResult.AsObject>,
    }
}

export enum EntryType {
    ALBUM = 0,
    SONG = 1,
    ARTIST = 2,
}
