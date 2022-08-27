// Copyright © Spelldawn 2021-present

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//    https://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#nullable enable

using System;
using System.Collections.Generic;
using System.Linq;
using Spelldawn.Game;
using Spelldawn.Protos;
using UnityEngine;

namespace Spelldawn.Assets
{
  [Serializable]
  public sealed class SpriteOverride
  {
    [SerializeField] string _substring = null!;
    public string Substring => _substring;

    [SerializeField] Sprite _sprite = null!;
    public Sprite Sprite => _sprite;
  }

  public sealed class DevelopmentAssets : MonoBehaviour
  {
    [SerializeField] Projectile _placeholderProjectile = null!;
    [SerializeField] TimedEffect _placeholderTimedEffect = null!;
    [SerializeField] Font _placeholderFont = null!;
    [SerializeField] List<SpriteOverride> _spriteOverrides = null!;

    public Sprite? GetSprite(SpriteAddress address)
    {
      return (
        from spriteOverride in _spriteOverrides
        where address.Address.Contains(spriteOverride.Substring)
        select spriteOverride.Sprite
      ).FirstOrDefault();
    }

    public Font GetFont(FontAddress address)
    {
      return _placeholderFont;
    }

    public Projectile GetProjectile(ProjectileAddress address)
    {
      return _placeholderProjectile;
    }

    public TimedEffect GetTimedEffect(EffectAddress address)
    {
      return _placeholderTimedEffect;
    }
  }
}