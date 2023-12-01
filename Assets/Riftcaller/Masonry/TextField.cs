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
using Riftcaller.Utils;
using UnityEngine.UIElements;

namespace Riftcaller.Masonry
{
  public static class TextFields
  {
    public static void Apply(Registry registry, NodeTextField field, TextFieldNode data)
    {
      field.SetGlobalIdentifierAndInitialText(data.GlobalIdentifier, data.InitialText);
      field.multiline = data.Multiline;
      field.isReadOnly = data.IsReadOnly;
      field.maxLength = (data.MaxLength > 0 ? (int)data.MaxLength : -1);
      field.isPasswordField = data.IsPasswordField;
      field.doubleClickSelectsWord = data.DoubleClickSelectsWord;
      field.tripleClickSelectsLine = data.TripleClickSelectsLine;
      field.maskChar = data.MaskCharacter?.Length > 0 ? data.MaskCharacter [0] : '*';
    }
  }

  public sealed class NodeTextField : TextField, INodeCallbacks
  {
    public VisualElement Self => this;
    readonly Lazy<Callbacks> _callbacks = new();
    public Lazy<Callbacks> Callbacks => _callbacks;
    public Node? Node { get; set; }
    string _globalIdentifier = "";

    public void SetGlobalIdentifierAndInitialText(string globalIdentifier, string initialText)
    {
      Errors.CheckArgument(globalIdentifier != "", "Global identifier cannot be empty");
      if (globalIdentifier != _globalIdentifier)
      {
        value = initialText;
        _globalIdentifier = globalIdentifier;
      }
    }
  }
}