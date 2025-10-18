const invoke = window.__TAURI__.invoke

export async function invokeConnect(server_ip) {
    return await invoke("connect_to_db", {server_ip: name});
}

