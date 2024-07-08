This is what I originally set out to do: make a sickle demo where the editor chrome was starting to
get hooked up to some actual functions, including the Bevy Remote Protocol (BRP), explore
asset-driven localization and how to manage inputs. In the very short term, I'm going to look at
serialization as it applies to sickle, to see if we can get the UI layouts themselves into assets,
and start hot-reloading our widgets. Next step from there is a widget editor that would allow
building the layout of an arbitrary, new editor widget and then serializing it to a file.

Well, the way these things work, mapping "event handlers" (actual events, observers, signals, etc)
from within a deflated widget to actual working systems defined elsewhere can be somewhat
straightforward. Just as we might add a SpawnMyWidget marker to a layout container and then have a
widget internal system scan for that marker and replace it with the actual widget bundle, so we can
also have a SpawnHandler or equivalent that describes which events should be handled in what way.
Then the system that handles them can react to its sudden presence in the ECS, and do what must be
done to wire things up behind the scenes.

This all depends on the editor framework having a basic way of providing access to its standard
components, so that's where we're going next. The basic idea is to expand on [how sickle itself
supports plugins](https://github.com/UmbraLuminosa/sickle_ui?tab=readme-ov-file#extending-sickle-ui)
and add some things like localization, input management, and other reactive properties.

In addition to basic UI editing capabilities, we would also like to implement the
[Caffeine Phase 1](https://hackmd.io/Oj7KqBOlRqGrFLxwyfYFCw) proposal for docks (activities).
Part of this involves adapting the popular
[Blender_bevy_components_workflow](https://github.com/kaosat-dev/Blender_bevy_components_workflow).
