import asyncio
import websockets

async def ws_handle_user(ws):
    async for message in ws:
        print(f"path: {message}")
        await ws.send("ok")

async def ws_main():
    async with websockets.serve(ws_handle_user, "0.0.0.0", 3333):
        await asyncio.Future()

asyncio.run(ws_main())
