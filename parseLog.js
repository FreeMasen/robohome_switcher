'use strict';
const fs = require('fs');
const moment = require('moment');
const logUpdate = require('log-update');

async function check() {
    let file = await getLogFile();
    let lines = parseLogFile(file);
    let errs = lines.filter(l => l.type === LineType.ERROR);
    let errMsg = errorMsg(errs);
    let counter = lines.filter(l => l.type === LineType.INFO &&
                                    !l.message.simple &&
                                    l.message.subject === MessageSubject.Counter);
    if (counter.length < 1) {
        return `No entries yet${errMsg}`;
    }
    let start = counter[0].date;
    let startMillis = start.milliseconds();
    let startSecs = start.seconds();
    let moved = counter.filter(l => l.date.millisecond() != startMillis || l.date.seconds() != startSecs);
    return movementMsg(moved, {s: startSecs, ms: startMillis});
}

function errorMsg(lines) {
    return lines.length < 1 ? '' : `\n${lines.length} errors\n----------${formatErrors(lines)}`;
}

function movementMsg(lines, start) {
    let toDisplay = lines.length > 10 ? lines.slice(lines.length - 10) : lines;
    return `${lines.length} counts moved\n----------${toDisplay.map(l => formatMovement(start, l))}`
}
/**
 *
 * @param {number} idx
 * @param {Object} start
 * @param {Line} line
 */
function formatMovement(start, line) {
    let movement = `${line.date.seconds() - start.s}s ${line.date.milliseconds() - start.ms}`
    return `\n${line.idx + 1}: ${line.location} ${movement}`;
}
/**
 *
 * @param {Array<Line>} errs
 * @returns {string}
 */
function formatErrors(errs) {
    return errs.map(l => `\n${l.idx}: ${l.location} - ${l.message.content}`)
}
/**
 *
 * @param {Moment} start
 * @param {Moment} end
 * @returns {string}
 */
function diff(start, end) {
    let ms = end.diff(start);
    let ret = '';
    if (ms < 1000) {
        return `${ms}ms`;
    }
    let seconds = 0;
    let minutes = 0;
    let hours = 0;
    let days = 0;
    let weeks = 0;
    let months = 0;
    let years = 0;
    while (ms >= 1000) {
        ms -= 1000;
        seconds += 1;
        if (seconds > 59) {
            seconds = 0;
            minutes += 1;
        }
        if (minutes > 59) {
            minutes = 0;
            hours += 1;
        }
        if (hours > 23) {
            hours = 0;
            days += 1;
        }
        if (days > 29) {
            days = 0;
            months += 1;
        }
        if (months > 11) {
            months = 0;
            years += 1;
        }
    }
    if (years > 0) {
        ret += `${years} years`;
    }
    if (months > 0) {
        ret += `${ret.length > 0 ? ' ' : ''}${months} months`;
    }
    if (days > 0) {
        ret += `${ret.length > 0 ? ' ' : ''}${days} days`;
    }
    if (hours > 0) {
        ret += `${ret.length > 0 ? ' ' : ''}${hours} hours`;
    }
    if (minutes > 0) {
        ret += `${ret.length > 0 ? ' ' : ''}${minutes}m`;
    }
    if (seconds > 0) {
        ret += `${ret.length > 0 ? ' ' : ''}${seconds}s`;
    }
    if (ms > 0) {
        ret += `${ret.length > 0 ? ' ' : ''}${ms}ms`;
    }
    return ret;
}
/**
 * @returns {Promise<string>} - The string buffer for this file
 */
function getLogFile() {
    return new Promise((res, rej) => {
        fs.readFile('out.log', {encoding: 'utf8'}, (err, data) => {
            if (err) return rej(err);
            if (typeof data !== 'string') data = data.toString();
            return res(data)
        });
    })
}

/**
 * Parse a log file text into an array of `Line`s
 *@param {string} text - The text to parse
 */
function parseLogFile(text) {
    return text.split('\n').map((l, i) => new Line(l, i));
}

class Line {
    /**
     * @param {string} text - The line to build from
     * @param {number} idx - The line number
     */
    constructor(text, idx) {
        this.original = text;
        this.idx = idx;
        let parts = text.split(' ');
        if (parts.length > 0 && parts[0] === '') parts.shift();
        switch (parts[0]) {
            case 'INFO':
                this.type = LineType.INFO;
            break;
            case 'DEBUG':
                this.type = LineType.DEBUG;
            break;
            case 'ERROR':
                this.type = LineType.ERROR;
            break;
            case 'WARN':
                this.type = LineType.WARN;
            break;
            default:
                this.type = LineType.UNKNOWN;
                this.message = new Message(parts);
                return;
        }
        this.date = moment(parts[1].substr(0, 20));
        this.location = parts[2];
        this.message = new Message(parts.splice(3));
    }
}

const LineType = Object.freeze({
    UNKNOWN: 'UNKNOWN',
    INFO: 'INFO',
    DEBUG: 'DEBUG',
    ERROR: 'ERROR',
    WARN: 'WARN',
});

const MessageSubject = Object.freeze({
    Unknown: 'Unknown',
    Counter: 'Counter',
    Scheduler: 'Scheduler',
    Flipper: 'Flipper',
    RabbitMQ: 'RabbitMQ',
});

const MessageDirection = Object.freeze({
    Incoming: 'Incoming',
    Outgoing: 'Outgoing',
})

class Message {
    constructor(parts) {
        if (parts.length === 1) {
            this.content = parts[0];
            this.simple = true;
            return;
        }
        switch (parts[0]) {
            case 'CT':
                this.subject = MessageSubject.Counter;
            break;
            case 'SC':
                this.subject = MessageSubject.Scheduler;
            break;
            case 'FL':
                this.subject = MessageSubject.Flipper;
            break;
            case 'MQ':
                this.subject = MessageSubject.RabbitMQ;
            break;
            case '??':
                this.subject = MessageSubject.Unknown;
            break;
            default:
                this.content = parts.join(' ');
                this.simple = true;
                return;
        }
        switch (parts[1]) {
            case 'IN':
                this.direction = MessageDirection.Incoming;
            break;
            case 'OUT':
                this.direction = MessageDirection.Outgoing;
            break;
            default:
                this.content = parts.join(' ');
                this.simple = true;
                return;
        }
        this.content = parts.slice(2).join(' ');
        this.simple = false;
    }
}
const dur = 1000;
function main() {
    check().then(msg => {
        logUpdate(`${moment().format('M/D/YY h:mm:ss a')}: ${msg}`);
    })
    .catch(e => {
        console.error(e);
        process.exit();
    })
}
main()
setInterval(main, dur);

async function getCounts() {
    let lines = parseLogFile(await getLogFile());
    let counts = lines.filter(l => l.type === LineType.INFO &&
                                !l.message.simple &&
                                l.message.subject === MessageSubject.Counter)
                        .map(l => l.original);
    fs.writeFile('counts.log', counts.join('\n'), err => {
        if (err) console.error(err);
        process.exit();
    });
}

// getCounts()