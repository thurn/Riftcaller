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

#nullable enable

using System;
using Riftcaller.Protos;
using Riftcaller.Services;
using UnityEngine;
using UnityEngine.UIElements;
using SliderDirection = Riftcaller.Protos.SliderDirection;

namespace Riftcaller.Masonry
{
  public static class Sliders
  {
    public static void Apply(Registry registry, Slider view, SliderNode data)
    {
      view.value = string.IsNullOrEmpty(data.PreferenceKey)
        ? data.InitialValue
        : PlayerPrefs.GetFloat(data.PreferenceKey);

      view.label = data.Label;

      view.direction = data.Direction switch
      {
        SliderDirection.Horizontal => UnityEngine.UIElements.SliderDirection.Horizontal,
        SliderDirection.Vertical => UnityEngine.UIElements.SliderDirection.Vertical,
        _ => UnityEngine.UIElements.SliderDirection.Horizontal
      };

      view.highValue = data.HighValue;
      view.lowValue = data.LowValue;

      view.inverted = data.Inverted;
      view.pageSize = data.PageSize;
      view.showInputField = data.ShowInputField;

      if (string.IsNullOrEmpty(data.PreferenceKey))
      {
        ((INodeCallbacks)view).SetCallback(Callbacks.Event.Change, null);
      }
      else
      {
        ((INodeCallbacks)view).SetCallback(Callbacks.Event.Change,
          () =>
          {
            PlayerPrefs.SetFloat(data.PreferenceKey, view.value);
            registry.SettingsService.SyncPreferences();
          });
      }

      Mason.ApplyStyle(registry,view.labelElement, data.LabelStyle);      
      Mason.ApplyStyle(registry, 
        view.Query(className: BaseSlider<float>.dragContainerUssClassName), data.DragContainerStyle);
      Mason.ApplyStyle(registry, 
        view.Query(className: BaseSlider<float>.trackerUssClassName), data.TrackerStyle);
      Mason.ApplyStyle(registry, 
        view.Query(className: BaseSlider<float>.draggerUssClassName), data.DraggerStyle);
      Mason.ApplyStyle(registry, 
        view.Query(className: BaseSlider<float>.draggerBorderUssClassName), data.DraggerBorderStyle);      
    }
  }

  public sealed class NodeSlider : Slider, INodeCallbacks
  {
    public VisualElement Self => this;
    readonly Lazy<Callbacks> _callbacks = new();
    public Lazy<Callbacks> Callbacks => _callbacks;
    public Node? Node { get; set; }
  }
}