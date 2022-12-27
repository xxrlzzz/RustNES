var ws = require("nodejs-websocket");
var port = 8099;

const SIGNAL_TYPE_JOIN = "join";
const SIGNAL_TYPE_LEAVE = "leave";
const SIGNAL_TYPE_NEW_PEER = "new-peer";
const SIGNAL_TYPE_PEER_LEAVE = "peer-leave";
const SIGNAL_TYPE_OFFER = "offer";
const SIGNAL_TYPE_ANSWER = "answer";
const SIGNAL_TYPE_ICE_CANDIDATE = "ice-candidate";

let users = new Map();
function handleJoin(userId, conn) {
  console.log("join", userId);
  if (users.has(userId)) {
    return;
  }
  if (users.size > 0) {
    for (let [key, value] of users) {
      value.send(
        JSON.stringify({
          type: SIGNAL_TYPE_NEW_PEER,
          userId: key,
          peerId: userId,
        })
      );
    }
  }
  users.set(userId, conn);
  conn.userId = userId;
}

function handleLeave(userId) {
  console.log("leave", userId);
  let conn = users[userId];
  if (conn && !conn.close) {
    conn.close = true;
    conn.close();
  } else {
    return;
  }
  users.delete(userId);
  users.forEach((user) => {
    user.send(
      JSON.stringify({
        type: SIGNAL_TYPE_PEER_LEAVE,
        userId: userId,
      })
    );
  });
}

function handleRemoteStreamInfo(message) {
  var userId = message.userId;
  var peerId = message.peerId;
  if (!users.has(userId) || !users.has(peerId)) {
    console.warn("Invalid user", userId, peerId);
    return;
  }
  var peer = users.get(peerId);
  peer.sendText(JSON.stringify(message));
  console.info(
    "Sent " + message.type + " message to " + peerId + " from " + userId
  );
}

var server = ws
  .createServer((conn) => {
    console.log("New connection");
    conn.on("text", (str) => {
      console.log("Received " + str);
      var signal = JSON.parse(str);
      switch (signal.type) {
        case SIGNAL_TYPE_JOIN:
          handleJoin(signal.userId, conn);
          break;
        case SIGNAL_TYPE_LEAVE:
          handleLeave(signal.userId);
          break;
        case SIGNAL_TYPE_OFFER:
        case SIGNAL_TYPE_ANSWER:
        case SIGNAL_TYPE_ICE_CANDIDATE:
          handleRemoteStreamInfo(signal);
          break;
        default:
          break;
      }
    });
    conn.on("close", (code, reason) => {
      console.log("Connection closed", conn.userId);
      handleLeave(conn.userId);
    });
    conn.on("error", function (err) {
      console.warn("Connection error " + err);
      handleLeave(conn.userId);
    });
  })
  .listen(port);
console.log("websocket server listening on port " + port);
