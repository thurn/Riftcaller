// Copyright © Riftcaller 2021-present

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//    https://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

using Riftcaller.Game;
using Riftcaller.Protos;
using UnityEngine;

#nullable enable

namespace Riftcaller.Services
{
  public sealed class RaidService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] ObjectDisplay _participants = null!;
    [SerializeField] float _roomsTopZ;
    [SerializeField] float _roomsBottomZ;
    [SerializeField] bool _raidActive;
    RoomIdentifier? _currentRoom;

    public ObjectDisplay RaidParticipants => _participants;

    public bool RaidActive
    {
      get => _raidActive;
      set
      {
        switch (value)
        {
          case true when !_raidActive:
            // Shift the raid display around based on room position in order to let weapons be visible
            transform.position = new Vector3(
              transform.position.x,
              transform.position.y,
              _registry.ArenaService.RoomsOnBottom ? _roomsBottomZ : _roomsTopZ);
            _registry.MusicService.SetMusicState(MusicState.Raid);
            _registry.ArenaService.LeftItems.SetGameContext(GameContext.ArenaRaidParticipant);
            break;
          case false when _raidActive:
            _registry.MusicService.SetMusicState(MusicState.Gameplay);
            _registry.ArenaService.LeftItems.SetGameContext(GameContext.Arena);
            break;
        }

        _registry.GameCharacterForPlayer(PlayerName.User).OnRaidStateChanged(value);
        _registry.GameCharacterForPlayer(PlayerName.Opponent).OnRaidStateChanged(value);
        _raidActive = value;
      }
    }
  }
}