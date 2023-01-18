const wsUri = "ws://localhost:7070/ws";

window.onload = function () {
    let conn = new WebSocket(wsUri);
    console.log("Connecting...");

    conn.onopen = function() {
        console.log("Connected.");
    }

    conn.onmessage = function(e) {
        console.log("Rec: " + e.data);
        document.getElementById("log").textContent =
            document.getElementById("log").textContent + "\n" + e.data;
    }

    conn.onclose = function() {
        console.log("closed.");
        conn = null;
    }

    function send() {
        conn.send(document.getElementById("input").value);
    }

    document.getElementById("btn").addEventListener("click", send);
}


