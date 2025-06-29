#!/usr/bin/env node

// Announce new OTA
// This script updates ota/{device}-{type}-{tags}.json which is used for checking update.

// Inputs:
//
// OTA_TIMESTAMP: timestamp of new OTA
// OTA_DEVICE: the device codename
// OTA_TYPE: the type of OTA
// OTA_PATH: path to the OTA package
// OTA_SIGNED: If the OTA is signed: 1, else it is considered unsigned
// OTA_BASE_URL: The base url for downloading the OTA
// OTA_OS_VERSION: OS version of this OTA

'use strict'

const fs = require('fs');
const path = require('path');
const { execFileSync } = require('child_process')

const scriptDir = __dirname;

function getOtaJsonPath(device, type, tags) {
    type = type.toLowerCase()
    const basename = `${device}-${type}-${tags}.json`
    return path.normalize(path.join(scriptDir, '..', 'ota', basename))
}

function readOtaJsonOrDefault(path) {
    let otaJson = {
        response: []
    }
    try {
        otaJson = JSON.parse(fs.readFileSync(path))
    } catch {
        console.warn("Warning: Failed to load ota json from", path)
    }
    return otaJson;
}

function writeOtaJson(path, payload) {
    fs.writeFileSync(path, JSON.stringify(payload, null, 2))
}

function newOTAEntry(otaPath, timestamp, url_base, os_version, release_type) {
    const otaFileName = path.basename(otaPath)
    const stats = fs.statSync(otaPath)
    // Don't calculate sha256sum using crypto because it only supports
    // file under 2GiB
    const stdout = execFileSync('sha256sum', [otaPath], {
        stdio: 'pipe',
        shell: false,
        windowsHide: true,
    })
    const sha256sum = stdout.toString().trim().split(/\s+/)[0]
    const id = sha256sum.slice(0, 32)
    return {
        "datetime": parseInt(timestamp),
        "filename": otaFileName,
        "id": id,
        "sha256": sha256sum,
        "romtype": release_type,
        "size": stats.size,
        "url": `${url_base}/${otaFileName}`,
        "version": os_version
    }
}

function checkInputs() {
    let pass = true;
    for(const input of ["OTA_DEVICE", "OTA_TYPE", "OTA_TIMESTAMP", "OTA_PATH", "OTA_OS_VERSION", "OTA_BASE_URL"]) {
        if (process.env[input] == null || process.env[input] === '') {
            console.error("Missing input environment variable", input)
            pass = false
        }
    }
    if(!pass) {
        process.exit(1)
    }
}

function main() {
    checkInputs()
    const signed = process.env.OTA_SIGNED === '1'
    const otaJsonPath = getOtaJsonPath(process.env.OTA_DEVICE, process.env.OTA_TYPE, signed ? "release-keys" : "test-keys")
    const otaJson = readOtaJsonOrDefault(otaJsonPath)
    const entry = newOTAEntry(process.env.OTA_PATH, process.env.OTA_TIMESTAMP, process.env.OTA_BASE_URL, process.env.OTA_OS_VERSION, process.env.OTA_TYPE);
    for(const oldEntry of otaJson.response) {
        if (oldEntry.id === entry.id) {
            console.error("FATAL: new OTA entry", entry, "conflicts with an old one: ", oldEntry);
            process.exit(1)
        }
    }
    otaJson.response.push(entry)
    writeOtaJson(otaJsonPath, otaJson)
}

main()
