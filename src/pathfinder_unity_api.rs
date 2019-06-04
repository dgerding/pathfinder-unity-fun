// This file is taken from revision 848eb0044353b735b9db2b1e1457b8e0930e2adf
// of the Pathfinder Project, and modified to suit the needs of the Unity
// plugin. Hopefully we will be able to merge changes upstream somehow so
// that this fork of the original doesn't need to exist.

// pathfinder/c/src/lib.rs
//
// Copyright © 2019 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! C bindings to Pathfinder.

use gl;
use pathfinder_canvas::{CanvasFontContext, CanvasRenderingContext2D, Path2D};
use pathfinder_geometry::basic::vector::{Vector2F, Vector2I};
use pathfinder_geometry::basic::rect::{RectF, RectI};
use pathfinder_geometry::color::ColorF;
use pathfinder_geometry::stroke::LineCap;
use pathfinder_gl::{GLDevice, GLVersion};
use pathfinder_gpu::resources::{FilesystemResourceLoader, ResourceLoader};
use pathfinder_gpu::{ClearParams, Device};
use pathfinder_renderer::concurrent::rayon::RayonExecutor;
use pathfinder_renderer::concurrent::scene_proxy::SceneProxy;
use pathfinder_renderer::gpu::renderer::{DestFramebuffer, Renderer};
use pathfinder_renderer::options::RenderOptions;
use pathfinder_renderer::scene::Scene;
use pathfinder_simd::default::F32x4;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};

// Constants

pub const PF_LINE_CAP_BUTT:   u8 = 0;
pub const PF_LINE_CAP_SQUARE: u8 = 1;
pub const PF_LINE_CAP_ROUND:  u8 = 2;

pub const PF_CLEAR_FLAGS_HAS_COLOR:   u8 = 0x1;
pub const PF_CLEAR_FLAGS_HAS_DEPTH:   u8 = 0x2;
pub const PF_CLEAR_FLAGS_HAS_STENCIL: u8 = 0x4;
pub const PF_CLEAR_FLAGS_HAS_RECT:    u8 = 0x8;

// Types

// `canvas`
pub type PFCanvasRef = *mut CanvasRenderingContext2D;
pub type PFPathRef = *mut Path2D;
pub type PFCanvasFontContextRef = *mut CanvasFontContext;
pub type PFLineCap = u8;

// `geometry`
#[repr(C)]
pub struct PFVector2F {
    pub x: f32,
    pub y: f32,
}
#[repr(C)]
pub struct PFVector2I {
    pub x: i32,
    pub y: i32,
}
#[repr(C)]
pub struct PFRectF {
    pub origin: PFVector2F,
    pub lower_right: PFVector2F,
}
#[repr(C)]
pub struct PFRectI {
    pub origin: PFVector2I,
    pub lower_right: PFVector2I,
}
#[repr(C)]
pub struct PFColorF {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

// `gl`
pub type PFGLDeviceRef = *mut GLDevice;
pub type PFGLVersion = GLVersion;
pub type PFGLFunctionLoader = extern "stdcall" fn(name: *const c_char, userdata: *mut c_void)
                                            -> *const c_void;
// `gpu`
pub type PFGLDestFramebufferRef = *mut DestFramebuffer<GLDevice>;
pub type PFGLRendererRef = *mut Renderer<GLDevice>;
// FIXME(pcwalton): Double-boxing is unfortunate. Remove this when `std::raw::TraitObject` is
// stable?
pub type PFResourceLoaderRef = *mut Box<dyn ResourceLoader>;
#[repr(C)]
pub struct PFClearParams {
    pub color: PFColorF,
    pub depth: f32,
    pub stencil: u8,
    pub rect: PFRectI,
    pub flags: PFClearFlags,
}
pub type PFClearFlags = u8;

// `renderer`
pub type PFSceneRef = *mut Scene;
pub type PFSceneProxyRef = *mut SceneProxy;
// TODO(pcwalton)
#[repr(C)]
pub struct PFRenderOptions {
    pub placeholder: u32,
}

// `canvas`

#[no_mangle]
pub unsafe extern "stdcall" fn PFCanvasCreate(font_context: PFCanvasFontContextRef,
                                        size: *const PFVector2F)
                                        -> PFCanvasRef {
    Box::into_raw(Box::new(CanvasRenderingContext2D::new(*Box::from_raw(font_context),
                                                         (*size).to_rust())))
}

#[no_mangle]
pub unsafe extern "stdcall" fn PFCanvasDestroy(canvas: PFCanvasRef) {
    drop(Box::from_raw(canvas))
}

#[no_mangle]
pub unsafe extern "stdcall" fn PFCanvasFontContextCreate() -> PFCanvasFontContextRef {
    Box::into_raw(Box::new(CanvasFontContext::new()))
}

#[no_mangle]
pub unsafe extern "stdcall" fn PFCanvasFontContextDestroy(font_context: PFCanvasFontContextRef) {
    drop(Box::from_raw(font_context))
}

/// Consumes the canvas.
#[no_mangle]
pub unsafe extern "stdcall" fn PFCanvasCreateScene(canvas: PFCanvasRef) -> PFSceneRef {
    Box::into_raw(Box::new(Box::from_raw(canvas).into_scene()))
}

#[no_mangle]
pub unsafe extern "stdcall" fn PFCanvasFillRect(canvas: PFCanvasRef, rect: *const PFRectF) {
    (*canvas).fill_rect((*rect).to_rust())
}

#[no_mangle]
pub unsafe extern "stdcall" fn PFCanvasStrokeRect(canvas: PFCanvasRef, rect: *const PFRectF) {
    (*canvas).stroke_rect((*rect).to_rust())
}

#[no_mangle]
pub unsafe extern "stdcall" fn PFCanvasSetLineWidth(canvas: PFCanvasRef, new_line_width: f32) {
    (*canvas).set_line_width(new_line_width)
}

#[no_mangle]
pub unsafe extern "stdcall" fn PFCanvasSetLineCap(canvas: PFCanvasRef, new_line_cap: PFLineCap) {
    (*canvas).set_line_cap(match new_line_cap {
        PF_LINE_CAP_SQUARE => LineCap::Square,
        PF_LINE_CAP_ROUND  => LineCap::Round,
        _                  => LineCap::Butt,
    });
}

/// Consumes the path.
#[no_mangle]
pub unsafe extern "stdcall" fn PFCanvasFillPath(canvas: PFCanvasRef, path: PFPathRef) {
    (*canvas).fill_path(*Box::from_raw(path))
}

/// Consumes the path.
#[no_mangle]
pub unsafe extern "stdcall" fn PFCanvasStrokePath(canvas: PFCanvasRef, path: PFPathRef) {
    (*canvas).stroke_path(*Box::from_raw(path))
}

#[no_mangle]
pub unsafe extern "stdcall" fn PFPathCreate() -> PFPathRef {
    Box::into_raw(Box::new(Path2D::new()))
}

#[no_mangle]
pub unsafe extern "stdcall" fn PFPathDestroy(path: PFPathRef) {
    drop(Box::from_raw(path))
}

#[no_mangle]
pub unsafe extern "stdcall" fn PFPathClone(path: PFPathRef) -> PFPathRef {
    Box::into_raw(Box::new((*path).clone()))
}

#[no_mangle]
pub unsafe extern "stdcall" fn PFPathMoveTo(path: PFPathRef, to: *const PFVector2F) {
    (*path).move_to((*to).to_rust())
}

#[no_mangle]
pub unsafe extern "stdcall" fn PFPathLineTo(path: PFPathRef, to: *const PFVector2F) {
    (*path).line_to((*to).to_rust())
}

#[no_mangle]
pub unsafe extern "stdcall" fn PFPathQuadraticCurveTo(path: PFPathRef,
                                                ctrl: *const PFVector2F,
                                                to: *const PFVector2F) {
    (*path).quadratic_curve_to((*ctrl).to_rust(), (*to).to_rust())
}

#[no_mangle]
pub unsafe extern "stdcall" fn PFPathBezierCurveTo(path: PFPathRef,
                                             ctrl0: *const PFVector2F,
                                             ctrl1: *const PFVector2F,
                                             to: *const PFVector2F) {
    (*path).bezier_curve_to((*ctrl0).to_rust(), (*ctrl1).to_rust(), (*to).to_rust())
}

#[no_mangle]
pub unsafe extern "stdcall" fn PFPathClosePath(path: PFPathRef) {
    (*path).close_path()
}

// `gl`

#[no_mangle]
pub unsafe extern "stdcall" fn PFFilesystemResourceLoaderLocate() -> PFResourceLoaderRef {
    let loader = Box::new(FilesystemResourceLoader::locate());
    Box::into_raw(Box::new(loader as Box<dyn ResourceLoader>))
}

#[no_mangle]
pub unsafe extern "stdcall" fn PFGLLoadWith(loader: PFGLFunctionLoader, userdata: *mut c_void) {
    gl::load_with(|name| {
        let name = CString::new(name).unwrap();
        loader(name.as_ptr(), userdata)
    });
}

#[no_mangle]
pub unsafe extern "stdcall" fn PFGLDeviceCreate(version: PFGLVersion, default_framebuffer: u32)
                                          -> PFGLDeviceRef {
    Box::into_raw(Box::new(GLDevice::new(version, default_framebuffer)))
}

#[no_mangle]
pub unsafe extern "stdcall" fn PFGLDeviceDestroy(device: PFGLDeviceRef) {
    drop(Box::from_raw(device))
}

#[no_mangle]
pub unsafe extern "stdcall" fn PFGLDeviceClear(device: PFGLDeviceRef, params: *const PFClearParams) {
    (*device).clear(&(*params).to_rust())
}

#[no_mangle]
pub unsafe extern "stdcall" fn PFResourceLoaderDestroy(loader: PFResourceLoaderRef) {
    drop(Box::from_raw(loader))
}

// `gpu`

#[no_mangle]
pub unsafe extern "stdcall" fn PFGLDestFramebufferCreateFullWindow(window_size: *const PFVector2I)
                                                             -> PFGLDestFramebufferRef {
    Box::into_raw(Box::new(DestFramebuffer::full_window((*window_size).to_rust())))
}

#[no_mangle]
pub unsafe extern "stdcall" fn PFGLDestFramebufferDestroy(dest_framebuffer: PFGLDestFramebufferRef) {
    drop(Box::from_raw(dest_framebuffer))
}

/// Takes ownership of `device` and `dest_framebuffer`, but not `resources`.
#[no_mangle]
pub unsafe extern "stdcall" fn PFGLRendererCreate(device: PFGLDeviceRef,
                                            resources: PFResourceLoaderRef,
                                            dest_framebuffer: PFGLDestFramebufferRef)
                                            -> PFGLRendererRef {
    Box::into_raw(Box::new(Renderer::new(*Box::from_raw(device),
                                         &**resources,
                                         *Box::from_raw(dest_framebuffer))))
}

#[no_mangle]
pub unsafe extern "stdcall" fn PFGLRendererDestroy(renderer: PFGLRendererRef) {
    drop(Box::from_raw(renderer))
}

#[no_mangle]
pub unsafe extern "stdcall" fn PFGLRendererGetDevice(renderer: PFGLRendererRef) -> PFGLDeviceRef {
    &mut (*renderer).device
}

#[no_mangle]
pub unsafe extern "stdcall" fn PFSceneProxyBuildAndRenderGL(scene_proxy: PFSceneProxyRef,
                                                      renderer: PFGLRendererRef,
                                                      options: *const PFRenderOptions) {
    (*scene_proxy).build_and_render(&mut *renderer, (*options).to_rust())
}

// `renderer`

#[no_mangle]
pub unsafe extern "stdcall" fn PFSceneDestroy(scene: PFSceneRef) {
    drop(Box::from_raw(scene))
}

#[no_mangle]
pub unsafe extern "stdcall" fn PFSceneProxyCreateFromSceneAndRayonExecutor(scene: PFSceneRef)
                                                                     -> PFSceneProxyRef {
    Box::into_raw(Box::new(SceneProxy::from_scene(*Box::from_raw(scene), RayonExecutor)))
}

#[no_mangle]
pub unsafe extern "stdcall" fn PFSceneProxyDestroy(scene_proxy: PFSceneProxyRef) {
    drop(Box::from_raw(scene_proxy))
}

// Helpers for `geometry`

impl PFColorF {
    #[inline]
    pub fn to_rust(&self) -> ColorF {
        ColorF(F32x4::new(self.r, self.g, self.b, self.a))
    }
}

impl PFRectF {
    #[inline]
    pub fn to_rust(&self) -> RectF {
        RectF::from_points(self.origin.to_rust(), self.lower_right.to_rust())
    }
}

impl PFRectI {
    #[inline]
    pub fn to_rust(&self) -> RectI {
        RectI::from_points(self.origin.to_rust(), self.lower_right.to_rust())
    }
}

impl PFVector2F {
    #[inline]
    pub fn to_rust(&self) -> Vector2F {
        Vector2F::new(self.x, self.y)
    }
}

impl PFVector2I {
    #[inline]
    pub fn to_rust(&self) -> Vector2I {
        Vector2I::new(self.x, self.y)
    }
}

// Helpers for `gpu`

impl PFClearParams {
    pub fn to_rust(&self) -> ClearParams {
        ClearParams {
            color: if (self.flags & PF_CLEAR_FLAGS_HAS_COLOR) != 0 {
                Some(self.color.to_rust())
            } else {
                None
            },
            rect: if (self.flags & PF_CLEAR_FLAGS_HAS_RECT) != 0 {
                Some(self.rect.to_rust())
            } else {
                None
            },
            depth: if (self.flags & PF_CLEAR_FLAGS_HAS_DEPTH) != 0 {
                Some(self.depth)
            } else {
                None
            },
            stencil: if (self.flags & PF_CLEAR_FLAGS_HAS_STENCIL) != 0 {
                Some(self.stencil)
            } else {
                None
            },
        }
    }
}

// Helpers for `renderer`

impl PFRenderOptions {
    pub fn to_rust(&self) -> RenderOptions {
        RenderOptions::default()
    }
}
