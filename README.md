# Mangui

Experimental GUI framework, inspired by DOM.

Most likely abandoned in favor of [cushy](https://github.com/khonsulabs/cushy) - it has a nice reactive model that just works, uses wgpu (more platforms than opengl) and doesn't need weird styling like here.

## Features

- uses Taffy for layouts - Grid and Flexbox support
- uses Femtovg as a renderer
  - currently runs on OpenGL (and OpenGL ES) only - no M1 support yet
  - no stroke dashing, custom shaders, 3d transforms or color fonts
    - stroke dashing could possibly be done using stroke pattern
- uses winit+glutin for window rendering
- events mirror their DOM counterparts (names, bubbling etc).
  - No capture part
  - Properties are changed for better usability
  - no currentTarget
  - no stopping propagation
  - no preventDefault as there are no default actions
- no layers support (yet :))

## Usage

Similar to DOM, there are Nodes, which are recursive. They implement a Node trait, which only requires to get styles (for layouting with Taffy), children and a draw function.

## Rusalka

Experimental 'svelte'-like framework for mangui.
Doesn't work very well yet.
