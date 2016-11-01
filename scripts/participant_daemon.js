/**
 * Randao participant daemon. Runs continuously monitoring a given Randao.
 * Participates in Randao by submitting commits and reveals when they are
 * required. Finally it will withdrawing any bounties at the end of a round.
 *
 * If a transaction fails with an error the script is terminated. This ensures
 * we don't keep sending transactions and draining our gas.
 */

'use strict'

const utils = require('./utils')
const RandaoStage = utils.RandaoStage

const path = require('path')
const FILENAME = path.basename(__filename)


/**
 * Randao participant deemon class. Provides just start() and stop().
 */

class ParticipantDaemon {

    /**
     * Constructor
     * @param web3 Connected instance of web3.js
     * @param randao Pudding abstraction for Randao.sol contract (truffle generated)
     * @param config Config properties
     * @param participantAddress Ethereum account address for the participant
     */

    constructor(web3, randao, config, participantAddress) {
        this.web3 = web3
        this.randao = randao
        this.config = config
        this.participantAddress = participantAddress
        this.logPrefix = `[${FILENAME}] [${participantAddress.substr(0, 6)}...]`
        this.committed = false
        this.revealed = false
    }

    /**
     * Start the daemon and set it to call doCheck every CHECK_INTERVAL milliseconds.
     */

    start() {
        this.intervalId = setInterval(this.doCheck, this.config.participantCheckInterval, this)
    }

    /**
     * Stop the daemon.
     */

    stop() {
        cancelInterval(this.intervalId)
    }

    /**
     * Check the current stage of randao. If it is inside a commit or reveal round
     * and we havn't yet submitted a value then go ahead and submit. Otherwise do
     * nothing.
     */

    doCheck(self) {
        const curBlk = self.web3.eth.blockNumber

        let cId, curCampaign
        self.randao.numCampaigns.call().then((numCampaigns) => {
            cId = numCampaigns - 1
            if (cId < 0) {
                self.log(`No campaigns yet ...`)
                return
            }

            self.randao.campaigns.call(cId).then((campaign) => {
                const stage = utils.getRandaoStage(curBlk, campaign[0], campaign[2], campaign[3])
                switch (stage) {
                    case RandaoStage.COMMIT:
                        if (self.committed == false) {
                            self.generateSecret(() => {
                                self.commit(self, cId)
                            })
                        }
                        break;

                    case RandaoStage.REVEAL:
                        if (self.committed == true && self.revealed == false) {
                            self.reveal(self, cId)
                        }
                        break;

                    case RandaoStage.WAITING_COMMIT:
                    case RandaoStage.FINISHED:
                        self.committed = self.revealed = false
                        break;

                    default:
                        self.logErr(`unknown stage: ${stage}`)
                        break;
                }
            })
        }).catch((e) => {
            self.logErr(`Error`, e)
            self.terminate()
        })
    }

    /**
     * Send commitment.
     */

    commit(self, cId) {
        self.log('sendTx commit() with commitment ' + self.commitment)
        self.randao.commit(cId, self.commitment, {
            from: self.participantAddress,
            value: self.config.deposit,
            gas: 1000000
        }).then((tx) => {
            self.log(`commit done (tx:${tx})\n`)
            self.committed = true
        }).catch((e) => {
            self.logErr(`commit failed`, e)
            self.terminate()
        })
    }

    /**
     * Send reveal.
     */

    reveal(self, cId) {
        self.log('sendTx reveal()')
        self.randao.reveal(cId, self.secret.toString(10), {
            from: self.participantAddress
        }).then((tx) => {
            self.log(`reveal done (tx:${tx})\n`)
            self.revealed = true
        }).catch((e) => {
            self.logErr(`reveal failed`, e)
            self.terminate()
        })
    }

    /**
     * Create secret and corresponding commitment
     */

    generateSecret(callback) {
        this.log('Generating secret for commit ...')
        const hexStr = utils.random32()
        this.secret = this.web3.toDecimal(`0x${hexStr}`)
        this.log(`secret dec: ${this.secret}`)

        let self = this
        // use contract sha3 as it produces a different value to web3.sha3 ..
        this.randao.shaCommit.call(this.secret.toString(10)).then((shaCommit) => {
            self.commitment = shaCommit
            self.log(`commitment: ${self.commitment}`)
            callback()
        })
    }

    /**
     * Logger routines
     */

    log(msg) {
        utils.log(`${this.logPrefix} ${msg}`)
    }

    logErr(msg, err) {
        let logMsg = `${this.logPrefix} ${msg}`
        if (err)
            logMsg += ` ERROR [${err}]`
        utils.log(logMsg, true)
    }

    /**
     * Exit the process - called when an exception has occured.
     */

    terminate() {
        this.log(`terminating!`)
        process.exit(-1)
    }
}

module.exports = ParticipantDaemon
