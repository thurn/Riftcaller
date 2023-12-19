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

namespace Riftcaller.Masonry
{
  public static class ScrollViews
  {
    public static void Apply(Registry registry, ScrollView view, ScrollViewNode data)
    {
      view.elasticity = data.Elasticity ?? 0.1f;
      view.horizontalPageSize = data.HorizontalPageSize ?? -1;
      Mason.ApplyStyle(registry, view.horizontalScroller, data.HorizontalScrollBar?.Style);
      view.horizontalScrollerVisibility = AdaptVisibility(data.HorizontalScrollBarVisibility);
      view.scrollDecelerationRate = data.ScrollDecelerationRate ?? 0.135f;
      view.touchScrollBehavior = data.TouchScrollBehavior switch
      {
        TouchScrollBehavior.Unspecified => ScrollView.TouchScrollBehavior.Clamped,
        TouchScrollBehavior.Unrestricted => ScrollView.TouchScrollBehavior.Unrestricted,
        TouchScrollBehavior.Elastic => ScrollView.TouchScrollBehavior.Elastic,
        TouchScrollBehavior.Clamped => ScrollView.TouchScrollBehavior.Clamped,
        _ => throw new ArgumentOutOfRangeException()
      };
      view.verticalPageSize = data.VerticalPageSize ?? -1;
      Mason.ApplyStyle(registry, view.verticalScroller, data.VerticalScrollBar?.Style);
      view.verticalScrollerVisibility = AdaptVisibility(data.VerticalScrollBarVisibility);
      view.mouseWheelScrollSize = data.MouseWheelScrollSize ?? 1.0f;
    }
    
    static ScrollerVisibility AdaptVisibility(ScrollBarVisibility visibility) =>
      visibility switch
      {
        ScrollBarVisibility.Unspecified => ScrollerVisibility.Auto,
        ScrollBarVisibility.Auto => ScrollerVisibility.Auto,
        ScrollBarVisibility.AlwaysVisible => ScrollerVisibility.AlwaysVisible,
        ScrollBarVisibility.Hidden => ScrollerVisibility.Hidden,
        _ => throw new ArgumentOutOfRangeException(nameof(visibility), visibility, null)
      };
  }
  
  public sealed class NodeScrollView : ScrollView, INodeCallbacks
  {
    public VisualElement Self => this;
    readonly Lazy<Callbacks> _callbacks = new();
    public Lazy<Callbacks> Callbacks => _callbacks;
    public Node? Node { get; set; }
  }  
}