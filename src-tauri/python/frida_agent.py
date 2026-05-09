import sys
import json
import frida

def list_devices():
    devices = []
    for device in frida.enumerate_devices():
        devices.append({
            "id": device.id,
            "name": device.name,
            "type": device.type
        })
    return devices

def run_script(device_id, package_name, script_content):
    device = frida.get_device(device_id) if device_id else frida.get_usb_device()
    session = device.attach(package_name)
    script = session.create_script(script_content)
    
    events = []
    def on_message(message, data):
        events.append({"message": message, "data": data})
        
    script.on('message', on_message)
    script.load()
    
    # Wait briefly to collect initial logs, in a real agent we might want streaming
    import time
    time.sleep(2)
    
    session.detach()
    return events

def handle_request(req):
    action = req.get("action")
    try:
        if action == "list_devices":
            return {"status": "ok", "data": list_devices()}
        elif action == "run_script":
            res = run_script(req.get("device_id"), req.get("package_name"), req.get("script_content"))
            return {"status": "ok", "data": res}
        else:
            return {"status": "error", "message": f"Unknown action: {action}"}
    except Exception as e:
        return {"status": "error", "message": str(e)}

if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--mcp":
        # Placeholder for full MCP server loop
        pass
    else:
        # Simple JSON RPC over stdio
        for line in sys.stdin:
            req = json.loads(line)
            res = handle_request(req)
            print(json.dumps(res))
            sys.stdout.flush()
