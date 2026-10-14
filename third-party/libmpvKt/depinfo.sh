#!/bin/bash -e

## Upstream versions. Every entry is a release tag; nothing tracks a branch.
## To bump: edit here, run ./download-deps.sh --refresh, run a full build, then CHANGELOG.md.
## GenerateBuildInfoTask reads the v_* lines, so the names below are part of the build contract.

v_ndk=29.0.14206865

v_mpv=0.41.0
v_ffmpeg=9.0.1
v_libass=0.17.5
v_libplacebo=7.360.1
v_dav1d=1.5.4
v_mbedtls=3.6.7
v_harfbuzz=14.4.0
v_freetype=2.14.3
v_fribidi=1.0.16
v_unibreak=7.0
v_lua=5.2.4

## Dependency tree. buildall.sh builds a target's dependencies first, in this order.

dep_mbedtls=()
dep_dav1d=()
dep_ffmpeg=(mbedtls dav1d)
dep_freetype2=()
dep_fribidi=()
dep_harfbuzz=()
dep_unibreak=()
dep_libass=(freetype2 fribidi harfbuzz unibreak)
dep_lua=()
dep_libplacebo=()
dep_mpv=(ffmpeg libass lua libplacebo)
dep_jni=(mpv)

## The cache identity of everything below mpv. CI keys its prefix cache on this string, so a
## bump of any dependency invalidates it, and a bump of mpv alone does not.
prefix_id="ndk${v_ndk}-ffmpeg${v_ffmpeg}-libass${v_libass}-placebo${v_libplacebo}-dav1d${v_dav1d}-mbedtls${v_mbedtls}-harfbuzz${v_harfbuzz}-freetype${v_freetype}-fribidi${v_fribidi}-unibreak${v_unibreak}-lua${v_lua}"
