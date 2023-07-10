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
using System.Linq;
using CustomizableCharacters;
using Spelldawn.Masonry;
using Spelldawn.Protos;
using Spelldawn.Services;
using Spelldawn.Utils;
using UnityEngine;

namespace Spelldawn.Common
{
  public static class CharacterAppearanceUtil
  {
    public static CharacterPreset BuildPreset(AssetService assets, CharacterAppearance appearance)
    {
      var preset = ScriptableObject.CreateInstance<CharacterPreset>();
      
      if (appearance.BodyColor != null)
      {
        preset.SetBodyColor(Mason.ToUnityColor(appearance.BodyColor));
      }
      
      preset.SetCustomizations(appearance.Customizations.Select(c => AdaptCustomization(assets, c)).ToArray());
      
      preset.SetScaleGroups(appearance.ScaleGroups.Select(AdaptScaleGroupPreset).ToArray());

      return preset;
    }

    static Customization AdaptCustomization(AssetService assets, CharacterAppearanceCustomization customization)
    {
      var result = new Customization(assets.GetCustomizationData(
        Errors.CheckNotNull(customization.Data, "Expected customization data address")));
      
      if (customization.Color != null)
      {
        result.SetMainColor(Mason.ToUnityColor(customization.Color));
      }
      
      result.SetDetailSpriteIndex((int)customization.DetailSpriteIndex);

      if (customization.DetailColor != null)
      {
        result.SetDetailColor(Mason.ToUnityColor(customization.DetailColor));
      }
      
      return result;
    }

    static ScaleGroupPreset AdaptScaleGroupPreset(CharacterAppearanceScaleGroup group)
    {
      var name = group.Name switch
      {
        CharacterScaleGroupName.Body => "Body",
        CharacterScaleGroupName.Head => "Head",
        CharacterScaleGroupName.Arms => "Arms",
        CharacterScaleGroupName.Hands => "Hands",
        CharacterScaleGroupName.Legs => "Legs",
        CharacterScaleGroupName.Feet => "Feet",
        CharacterScaleGroupName.Weapon => "Weapon",
        CharacterScaleGroupName.Shield => "Shield",
        _ => throw new ArgumentOutOfRangeException()
      };

      return new ScaleGroupPreset(name, group.Scale, group.Width, group.Length);
    }
  }
}