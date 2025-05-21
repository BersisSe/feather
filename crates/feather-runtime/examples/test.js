const ws = new WebSocket('ws://127.0.0.1:6060');

while (true){
    ws.onmessage = function (e) { console.log(e.data) };    
}
