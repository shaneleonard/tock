
// Set advertisement rate to be once per second. This must be set before
// including the bleno library (which is included from eddystone-beacon).
process.env['BLENO_ADVERTISING_INTERVAL'] = 1000;

var http       = require('http');
var os         = require('os');

var bleno      = require('bleno');
var debug      = require('debug')('http-gateway');
var httpparser = require('http-string-parser');
var request = require('request');


// var req = 'GET https://j2x.us/\n\
// host: j2x.us';

// console.log(req);

// var a = httpparser.parseRequest(req);
// console.log(a);

// if (!a.uri.startsWith('h')) {
// 	a.uri = 'http://' + a.headers.host + a.uri;
// }

// request(a, function (error, response, body) {
// 	console.log('error:', error);
// 	console.log('statusCode:', response && response.statusCode);
// 	console.log('body:', body);
// });

var DEVICE_NAME         = 'http-gateway';
var SERVICE_UUID        = '16ba0001cf44461eb8894f9a90f6b330';
var CHARACTERISTIC_UUID = '16ba0002cf44461eb8894f9a90f6b330';


bleno.on('stateChange', function(state) {
	debug('on -> stateChange: ' + state);

	if (state === 'poweredOn') {
		bleno.startAdvertising(DEVICE_NAME, [SERVICE_UUID], function (err) {
			if (err) {
				console.log('could not start advertising.');
				console.log(err);
			}
		});
	} else {
		bleno.stopAdvertising();
	}
});

bleno.on('advertisingStart', function(error) {
	debug('on -> advertisingStart: ' + (error ? 'error ' + error : 'success'));

	if (!error) {
		console.log('setup services')
		bleno.setServices([
			new bleno.PrimaryService({
				uuid: SERVICE_UUID,
				characteristics: [
					new bleno.Characteristic({
						uuid: CHARACTERISTIC_UUID,
						properties: ['read', 'write', 'writeWithoutResponse'],
						onReadRequest: function (offset, callback) {
							if (offset == 0) {
								debug('Read characteristic.');
							}

							callback(bleno.Characteristic.RESULT_SUCCESS, 'abc');
						},
						onWriteRequest: function(data, offset, withoutResponse, callback) {
							console.log('got write')
							console.log(data)

							var msg = data.toString('utf-8');
							console.log(msg);
							// var b = httpparser.parseRequest(msg);
							// console.log(b);

							// if (!b.uri.startsWith('h')) {
							// 	b.uri = 'http://' + b.headers.host + b.uri;
							// }

							// request(b, function (error, response, body) {
							// 	console.log('error:', error);
							// 	console.log('statusCode:', response && response.statusCode);
							// 	console.log('body:', body);
							// });

							callback(bleno.Characteristic.RESULT_SUCCESS)
						},
						onSubscribe: function (maxValueSize, updateValueCallback) {
							debug('subscribe characteristic');
							_notify_callback = updateValueCallback;
						},
						onUnsubscribe: function () {
							_notify_callback = null;
						}
					})
				]
			})
		], function (error2) {
			if (error2) {
				console.log('error creating services');
				console.log(error2);
			}
		});
	}
});

bleno.on('advertisingStartError', function (err) {
	debug('adv start err: ' + err);
	debug(err);
});

bleno.on('connect', function (client_address) {
	debug('connect ' + client_address);
});

bleno.on('disconnect', function (client_address) {
	debug('DISCONNECT ' + client_address);
});

bleno.on('advertisingStop', function () {
	debug('ADV STOPPED');
});

