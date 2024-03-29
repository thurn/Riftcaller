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

using System;
using System.Collections;
using Riftcaller.Protos;
using Riftcaller.Utils;
using UnityEngine;

#nullable enable

namespace Riftcaller.Game
{
  public sealed class Room : MonoBehaviour
  {
    [SerializeField] RectangularObjectDisplay _inRoom = null!;

    public ObjectDisplay BackCards => _inRoom;

    // Defenders are sorted in order, index 0 represents the rearmost defender
    [SerializeField] RectangularObjectDisplay _defenders = null!;

    public ObjectDisplay FrontCards => _defenders;

    [SerializeField] RoomIdentifier _roomId;
    public RoomIdentifier RoomId => _roomId;

    [SerializeField] SpriteRenderer _spriteRenderer = null!;
    public SpriteRenderer SpriteRenderer => _spriteRenderer;

    void Start()
    {
      Errors.CheckNotNull(_inRoom, $"Missing inRoom for {name}");
      Errors.CheckNotNull(_defenders, $"Missing defenders for {name}");
      Errors.CheckState(_roomId != RoomIdentifier.Unspecified, $"RoomId missing for {name}");
      _spriteRenderer = ComponentUtils.GetComponent<SpriteRenderer>(this);
    }

    public ObjectDisplay ObjectDisplayForLocation(ClientRoomLocation location) => location switch
    {
      ClientRoomLocation.Back => _inRoom,
      ClientRoomLocation.Front => _defenders,
      _ => throw new ArgumentOutOfRangeException(nameof(location), location, null)
    };

    public IEnumerator AddCard(Displayable card, ClientRoomLocation location, bool animate) => location switch
    {
      ClientRoomLocation.Back => _inRoom.AddObject(card, animate),
      _ => _defenders.AddObject(card, animate)
    };
  }
}