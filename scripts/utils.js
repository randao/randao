'use strict'

const crypto = require('crypto');

const RandaoStage = {
    WAITING_COMMIT: 1,
    COMMIT: 2,
    REVEAL: 3,
    FINISHED: 4,

    name: (num) => {
        for (const key of Object.keys(RandaoStage)) {
            if (RandaoStage[key] == num)
                return key
        }
    }
}

/**
 * Determine current stage of Randao.
 *
 * @param curBlk Current block number on Ethereum
 * @param endBlk Randao end block
 * @param balkline Campaign balkline
 * @param deadline Campaign deadline
 * @return corresponding stage
 */

function getRandaoStage(curBlk, endBlk, balkline, deadline) {
    let stage

    if (curBlk > endBlk)
        stage = RandaoStage.FINISHED
    else if (curBlk >= endBlk - deadline)
        stage = RandaoStage.REVEAL
    else if (curBlk >= endBlk - balkline)
        stage = RandaoStage.COMMIT
    else
        stage = RandaoStage.WAITING_COMMIT

    return stage
}


/**
 * Generate a 32 byte secret number using crypto randomBytes.
 */

function random32() {
    const buf = crypto.randomBytes(32);
    const hexStr = buf.toString('hex')
    return hexStr
}


/**
 * Log with a timestamp
 */

function log(msg, error) {
    const logStr = `[${new Date().toISOString()}] ${msg}`
    if (error && error == true)
        console.error(logStr)
    else
        console.log(logStr)
}


module.exports = {
    RandaoStage,
    getRandaoStage,
    random32,
    log
}
