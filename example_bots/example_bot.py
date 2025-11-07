from dataclasses import dataclass
from typing import Any, Callable, Coroutine
import asyncio
import json


# MsgFromPlayerToLobby
@dataclass
class Connect:
    name: str
    uuid: str


@dataclass
class Disconnect:
    pass


MsgFromPlayerToLobby = Connect | Disconnect


# MsgFromLobbyToPlayer
@dataclass
class Unconnected:
    pass


@dataclass
class UnableToSendMsgToGame:
    pass


@dataclass
class AlreadyConnected:
    pass


MsgFromLobbyToPlayer = Unconnected | UnableToSendMsgToGame | AlreadyConnected


# MessageFromPlayerToGame
@dataclass
class IThinkThereAre:
    number: int  # 0..256
    face: int  # 1..6


@dataclass
class Liar:
    pass


@dataclass
class Exactly:
    pass


MsgFromPlayerToGame = IThinkThereAre | Liar | Exactly


# MessageFromGameToPlayer
@dataclass
class Connected:
    number_of_players: int  # 0..usize::max
    already_connected: int  # 0..usize::max


@dataclass
class GameStarted:
    pass


@dataclass
class GameEnded:
    winner: str


@dataclass
class Reconnected:
    pass


@dataclass
class GameAlreadyStarted:
    pass


@dataclass
class UnkownMessage:
    message: str


@dataclass
class NotYourTurn:
    pass


@dataclass
class Turn:
    turn: int  # 0..usize::max (u64::max)
    number: int  # 0..256
    face: int  # 1..6
    next_player: str


@dataclass
class Round:
    result: str
    revealed_hands: dict[str, list[int]]


@dataclass
class YouRolled:
    hand: list[int]


MsgFromGameToPlayer = (
    Connected
    | GameStarted
    | GameEnded
    | Reconnected
    | GameAlreadyStarted
    | UnkownMessage
    | Turn
    | Round
    | YouRolled
)

MsgOut = MsgFromPlayerToLobby | MsgFromPlayerToGame
MsgIn = MsgFromLobbyToPlayer | MsgFromGameToPlayer


def serialize(o: object):
    return {o.__class__.__name__: o.__dict__}


def deserialize(s: str) -> MsgIn:
    msg = json.loads(s)
    # print("incoming msg:", msg)
    class_name: str
    payload: Any
    if isinstance(msg, str):
        class_name = msg
        payload = {}
    elif isinstance(msg, dict):
        class_name = next(iter(msg))
        payload = msg[class_name]
    else:
        return UnkownMessage(f"Client side: The `{msg}` is not str or dict!")
    # print("incoming class_name:", class_name)
    for cls in MsgIn.__args__:
        if cls.__name__ == class_name:
            return cls(**payload)
    return UnkownMessage(f"Client side: class `{class_name}` not found in MsgIn!")


class JsonLineTCPClient:
    def __init__(
        self,
        host: str,
        port: int,
        handle_incomming: Callable[[MsgIn], Coroutine[Any, Any, None]],
    ):
        self.host = host
        self.port = port
        self.handle_incomming = handle_incomming

    async def connect(self) -> None:
        self.reader, self.writer = await asyncio.open_connection(self.host, self.port)
        self._recv_task = asyncio.create_task(self._receive_loop())
        await self.let_receive()

    async def send(self, msg: MsgOut) -> None:
        if self.writer is None:
            raise RuntimeError("Not connected")
        data = json.dumps(msg, default=serialize)
        print("Sending:", data)
        self.writer.write((data + "\n").encode("utf-8"))
        await self.writer.drain()
        await self.let_receive()

    async def _receive_loop(self) -> None:
        assert self.reader is not None
        while True:
            try:
                line = await self.reader.readline()
                if not line:
                    print("Connection closed by server")
                    break
                data = line.decode("utf-8").rstrip("\n")
                # print("incoming data:", data)
                msg = deserialize(data)
                # print("incoming msg:", msg)
                await self.handle_incomming(msg)

            except asyncio.CancelledError:
                break
            except Exception as e:
                print("Receive error:", e)
                break

    async def let_receive(self) -> None:
        await asyncio.sleep(0)


class Bot:
    def __init__(self) -> None:
        self.state = "Conneting"
        self.hand = []
        self.current_number = 0
        self.current_face = 0

    async def run(self) -> None:
        # Connect to TCP Server
        client = JsonLineTCPClient("127.0.0.1", 5942, self.handle_incomming)
        await client.connect()

        # Connect to Game
        await client.send(Connect("PyBot", "80f2fa9e-5fbd-4e73-a518-141cb0e1e2d5"))

        # Game Loop
        while True:
            # TODO: Handle server closed
            match self.state:
                case "MyTurn":
                    await client.send(
                        IThinkThereAre(
                            self.current_number + 1,
                            self.current_face if self.current_face != 0 else 6,
                        )
                    )
                    self.state = "Wait"
                case _:
                    await asyncio.sleep(1)

    async def handle_incomming(self, msg: MsgIn):
        print("Incoming Msg:", msg)
        if isinstance(msg, Connect | AlreadyConnected):
            self.state = "Connected"
        elif isinstance(msg, GameStarted):
            self.state = "GameStarted"
        elif isinstance(msg, YouRolled):
            self.hand = msg.hand
        elif isinstance(msg, Turn):
            if msg.next_player == "PyBot":
                self.state = "MyTurn"
            else:
                self.state = "OtherPlayersTurn"
            self.current_number = msg.number
            self.current_face = msg.face


if __name__ == "__main__":
    bot = Bot()
    asyncio.run(bot.run())
