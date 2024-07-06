## Purpose

This project in its current state is an attempt to wire up sickle_ui to an actually working app.

This has largely been successful. Bevy 0.14.0 + sickle + bevy_fluent + (slight fork of) bevy_remote
can rotate a camera in the editor window, and in a second window, an instance of the server example
will display the same scene. When the editor camera moves, the server camera receives its updated
transform via BRP, and updates its viewport to match. It is also possible to toggle an FPS counter
widget in the server window from within the editor UI. All of this is also translated into French,
which may be selected from a dropdown in the upper right corner of the UI.

I'm calling it beverage because that is the first code name I thought of a couple months ago when
I set out to explore the current state of Bevy. It ties into a sort of cross-engine abstraction
I was working on before that. I won't go into detail, but the point is that beverage is more of a
platform than an outright product. In this essay I will describe the concept and then try to be as
practical as possible with technical details and probably end with a slightly unhinged rant about
something or other.

## The Concept

There is definitely a Bevy Way(tm), and it's the sort of thing that means many things to many people.
While I am not by any stretch a formalist, I am a little astounded that I have yet to find an
archived flamewar where people begin their posts with "as we have shown in the ECS calculus."

There are certainly patterns, some documented, and some like Observers are so common they got
turned into a new thing. There's the cheatbook which has many clues, and there's the vibrant and
friendly Discord where you can quickly learn how to do things, and more importantly, how far
from the Way you have already been tempted to stray.

## In The ECS Calculus

I won't presume to be able to scratch the surface, so I'll try to stick to things that are
immediately helpful. The idea behind beverage is to build an SDK. This means a barebones structure
that wraps the dependencies and a lightweight mechanism to expose them as services to every editor
plugin that wishes to use them. The kind of structure I'm picturing is a little bit different than
many of the code examples I've seen in the past few months. Let's start with an example: localization.

TODO: explain how bevy_fluent works, how it's currently integrated, and how we wish to see the
integration enhanced.

TODO: link to the caffeine documentation
