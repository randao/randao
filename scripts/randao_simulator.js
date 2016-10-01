/**
 * Randao simulator. Simulates rounds of Randao with scripts acting as Randao
 * participants. This is fully automated with no human actors.
 */

'use strict'

const config = {
    rpc: 'http://localhost:4500',
    deposit: 1000,
}

const Web3 = require('web3')
const Randao = require(`../build/contracts/Randao.sol.js`)
const CampaignCreatorDaemon = require('./campaign_creator_daemon')
const ParticipantDaemon = require('./participant_daemon')

const web3Provider = new Web3.providers.HttpProvider(config.rpc)
const web3 = new Web3(web3Provider)
console.log(`web3 connected: ${web3.isConnected()}\n`)
web3.eth.defaultAccount = web3.eth.accounts[0]

Randao.setProvider(web3.currentProvider)
const randao = Randao.deployed()

const creator = new CampaignCreatorDaemon(web3, randao, config)
creator.start()

const p1Addr = web3.eth.accounts[1]
const p1 = new ParticipantDaemon(web3, randao, config, p1Addr)
p1.start()

const p2Addr = web3.eth.accounts[2]
const p2 = new ParticipantDaemon(web3, randao, config, p2Addr)
p2.start()
