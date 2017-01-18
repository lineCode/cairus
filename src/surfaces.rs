/*
 * Cairus - a reimplementation of the cairo graphics library in Rust
 *
 * Copyright © 2017 CairusOrg
 *
 * This library is free software; you can redistribute it and/or
 * modify it either under the terms of the GNU Lesser General Public
 * License version 2.1 as published by the Free Software Foundation
 * (the "LGPL") or, at your option, under the terms of the Mozilla
 * Public License Version 2.0 (the "MPL"). If you do not alter this
 * notice, a recipient may use your version of this file under either
 * the MPL or the LGPL.
 *
 * You should have received a copy of the LGPL along with this library
 * in the file LICENSE-LGPL-2_1; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Suite 500, Boston, MA 02110-1335, USA
 * You should have received a copy of the MPL along with this library
 * in the file LICENSE-MPL-2_0
 *
 * The contents of this file are subject to the Mozilla Public License
 * Version 2.0 (the "License"); you may not use this file except in
 * compliance with the License. You may obtain a copy of the License at
 * http://www.mozilla.org/MPL/
 *
 * This software is distributed on an "AS IS" basis, WITHOUT WARRANTY
 * OF ANY KIND, either express or implied. See the LGPL or the MPL for
 * the specific language governing rights and limitations.
 *
 * The Original Code is the cairus graphics library.
 *
 * Contributor(s):
 *  Bobby Eshleman <bobbyeshleman@gmail.com>
 *
 */

use std::slice::IterMut;
use types::Rgba;

struct ImageSurface {
    base: Vec<Rgba>,
    width: usize,
    height: usize,
}

impl ImageSurface {
    fn create(width: usize, height: usize) -> ImageSurface {
        let base = vec![Rgba::new(0., 0., 0., 0.); width * height];
        ImageSurface::from_vec(base, width, height)
    }

    fn iter(&self) -> ImageSurfaceRefIterator {
        ImageSurfaceRefIterator{surface: self, index: 0}
    }

    fn iter_mut(&mut self) -> IterMut<Rgba> {
        self.base.iter_mut()
    }

    fn from_vec(vec: Vec<Rgba>, width: usize, height: usize) -> ImageSurface {
        ImageSurface {
            base: vec,
            width: width,
            height: height,
        }
    }
}

impl IntoIterator for ImageSurface {
    type Item = Rgba;
    type IntoIter = ImageSurfaceIterator;

    fn into_iter(self) -> Self::IntoIter {
        ImageSurfaceIterator{surface: self.base, index: 0, width: self.width, height: self.height}
    }
}

/*
impl FromIterator<Rgba> for ImageSurface {
    fn from_iter<I: IntoIterator<Item = Rgba>>(iter: I) -> Self {
        let vec = Vec::with_capacity(iter.width * iter.height);
        for pixel in iter{
            vec.push(pixel);
        }
        ImageSurface::from_vec(vec, iter.width, iter.height);
    }
}
*/

struct ImageSurfaceIterator {
    surface: Vec<Rgba>,
    index: usize,
    width: usize,
    height: usize,
}

impl Iterator for ImageSurfaceIterator {
    type Item = Rgba;

    fn next(&mut self) -> Option<Rgba> {
        match self.index < self.surface.len() {
            true => {
                let elem = self.surface[self.index];
                self.index += 1;
                Some(elem)
            },
            false => None
        }
    }
}

struct ImageSurfaceRefIterator<'a> {
    surface: &'a ImageSurface,
    index: usize,
}

impl<'a> Iterator for ImageSurfaceRefIterator<'a> {
    type Item = &'a Rgba;

    fn next(&mut self) -> Option<Self::Item> {
        match self.index < self.surface.width * self.surface.height {
            true => {
                let result = Some(&self.surface.base[self.index]);
                self.index += 1;
                result
            },
            false => None,
        }
    }
}


trait IntoSurface {
    fn into_surface(self) -> ImageSurface;
}


#[cfg(test)]
mod tests {
    use types::Rgba;
    use surfaces::ImageSurface;
    use operators::{Operator, fetch_operator};

    #[test]
    fn test_image_surface_new() {
        // Test that ImageSurface's IntoIterator is functioning correctly
        let default_rgba = Rgba::new(0., 0., 0., 0.);
        let surface = ImageSurface::create(100, 100);
        for pixel in surface.base {
            assert_eq!(pixel, default_rgba);
        }
    }

    #[test]
    fn test_image_surface_into_iter() {
        // Test that ImageSurface's IntoIterator is functioning correctly
        let default_rgba = Rgba::new(0., 0., 0., 0.);
        let surface = ImageSurface::create(100, 100);
        for pixel in surface.into_iter() {
            assert_eq!(pixel, default_rgba);
        }
    }

    #[test]
    fn test_image_surface_iter() {
        // Passes if ImageSurface::iter() functions properly
        let surface = ImageSurface::create(100, 100);

        // Leave pixel.red to default (0.0), change all other hcannels to 1.0
        let result = surface
                        .iter()
                        .map( |&pixel| Rgba{red: pixel.red, green: 1., blue: 1., alpha: 1.})
                        .collect::<Vec<Rgba>>();

        let expected = Rgba{red: 0., green: 1., blue: 1., alpha: 1.};
        for pixel in result.into_iter() {
            // Red is 0. because it is the default, the others got set to 1.
            assert_eq!(pixel, expected);
        }
    }

    #[test]
    fn test_image_surface_iter_mut() {
        // Passes if ImageSurface::iter_mut() functions properly
        let mut surface = ImageSurface::create(100, 100);

        for mut pixel in surface.iter_mut() {
            // Red is 0. because it is the default, the others got set to 1.
            //pixel = Rgba::new(0.5, 0.5, 0.5, 0.5);
            pixel.red = 1.;
        }

        let expected = Rgba::new(1., 0., 0., 0.);
        for pixel in surface {
            assert_eq!(pixel, expected);
        }
    }

    #[test]
    fn test_image_surface_with_operator() {
        // Demonstrates usage with an operator

        // Create our source Rgba, destination, and choose an operator
        let source_rgba = Rgba::new(1., 1., 1., 1.);
        let mut destination = ImageSurface::create(100, 100);
        let op = Operator::Over;

        // Using fetch_operator and the Operator enum.
        let operator = fetch_operator(&op);
        for mut pixel in destination.iter_mut() {
            operator(&source_rgba, pixel);
        }

        let expected = Rgba::new(1., 1., 1., 1.);
        for pixel in destination {
            assert_eq!(pixel, expected);
        }
    }
}
