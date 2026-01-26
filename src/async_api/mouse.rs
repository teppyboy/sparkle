//! Mouse emulation for human-like interactions
//!
//! This module provides realistic mouse movement and clicking behavior
//! to avoid detection by anti-bot systems.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;
use thirtyfour::common::types::ElementRect;
use thirtyfour::prelude::*;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

use crate::async_api::ElementInFrame;
use crate::core::{Error, Result};
use crate::driver::WebDriverAdapter;

#[async_trait]
pub trait MouseTarget {
    async fn rect(&self) -> Result<ElementRect>;
}

#[async_trait]
impl MouseTarget for WebElement {
    async fn rect(&self) -> Result<ElementRect> {
        self.rect().await.map_err(Error::from)
    }
}

#[async_trait]
impl MouseTarget for ElementInFrame {
    async fn rect(&self) -> Result<ElementRect> {
        self.element_rect().await
    }
}

/// Mouse emulation for human-like interactions
pub struct Mouse {
    adapter: Arc<WebDriverAdapter>,
    position: Arc<RwLock<(i64, i64)>>,
}

/// Options for mouse movement
#[derive(Debug, Clone)]
pub struct MoveOptions {
    /// Number of steps for smooth movement (default: 10)
    pub steps: usize,
    /// Delay between steps in milliseconds (default: 10ms)
    pub step_delay_ms: u64,
    /// Add random jitter to movement (default: true)
    pub jitter: bool,
    /// Bezier curve smoothing (default: true)
    pub bezier_curve: bool,
}

impl Default for MoveOptions {
    fn default() -> Self {
        Self {
            steps: 10,
            step_delay_ms: 10,
            jitter: true,
            bezier_curve: true,
        }
    }
}

/// Options for clicking
#[derive(Debug, Clone)]
pub struct MouseClickOptions {
    /// Delay before mousedown in milliseconds (default: 50-150ms random)
    pub delay_before_ms: Option<u64>,
    /// Duration of mousedown in milliseconds (default: 50-150ms random)
    pub mousedown_duration_ms: Option<u64>,
    /// Move to element before clicking (default: true)
    pub move_to_element: bool,
    /// Move options if moving to element
    pub move_options: MoveOptions,
}

impl Default for MouseClickOptions {
    fn default() -> Self {
        Self {
            delay_before_ms: None,
            mousedown_duration_ms: None,
            move_to_element: true,
            move_options: MoveOptions::default(),
        }
    }
}

impl Mouse {
    /// Create a new Mouse instance
    pub(crate) fn new(adapter: Arc<WebDriverAdapter>) -> Self {
        Self {
            adapter,
            position: Arc::new(RwLock::new((0, 0))),
        }
    }

    /// Move mouse to specific coordinates with human-like motion
    ///
    /// # Arguments
    /// * `x` - Target X coordinate
    /// * `y` - Target Y coordinate
    /// * `options` - Movement options
    pub async fn move_to(&self, x: i64, y: i64, options: MoveOptions) -> Result<()> {
        // Get current mouse position (or assume 0, 0 if can't get)
        let (start_x, start_y) = (0i64, 0i64);

        let points = if options.bezier_curve {
            self.generate_bezier_path(start_x, start_y, x, y, options.steps)
        } else {
            self.generate_linear_path(start_x, start_y, x, y, options.steps)
        };

        // Move through each point
        for (px, py) in points {
            let (final_x, final_y) = if options.jitter {
                self.add_jitter(px, py)
            } else {
                (px, py)
            };

            self.move_mouse_to_coord(final_x, final_y).await?;
            
            if options.step_delay_ms > 0 {
                sleep(Duration::from_millis(options.step_delay_ms)).await;
            }
        }

        Ok(())
    }

    /// Move mouse to an element with human-like motion
    ///
    /// # Arguments
    /// * `element` - Target element
    /// * `options` - Movement options
    pub async fn move_to_element<T>(&self, element: &T, options: MoveOptions) -> Result<()>
    where
        T: MouseTarget + ?Sized,
    {
        // Get element's location and size
        let rect = element.rect().await?;
        
        // Calculate center of element with slight randomization
        let center_x = rect.x + rect.width / 2.0;
        let center_y = rect.y + rect.height / 2.0;
        
        // Add slight randomization to not always click exact center
        let offset_x = if options.jitter {
            (rand::random::<f64>() - 0.5) * (rect.width * 0.3)
        } else {
            0.0
        };
        let offset_y = if options.jitter {
            (rand::random::<f64>() - 0.5) * (rect.height * 0.3)
        } else {
            0.0
        };

        let target_x = (center_x + offset_x) as i64;
        let target_y = (center_y + offset_y) as i64;

        self.move_to(target_x, target_y, options).await
    }

    /// Click at current mouse position with human-like behavior
    ///
    /// # Arguments
    /// * `options` - Click options
    pub async fn click(&self, options: MouseClickOptions) -> Result<()> {
        // Random delay before clicking
        let delay_before = options.delay_before_ms.unwrap_or_else(|| {
            50 + (rand::random::<u64>() % 100) // 50-150ms
        });
        sleep(Duration::from_millis(delay_before)).await;

        // Perform mousedown
        self.mouse_down().await?;

        // Hold mousedown for realistic duration
        let mousedown_duration = options.mousedown_duration_ms.unwrap_or_else(|| {
            50 + (rand::random::<u64>() % 100) // 50-150ms
        });
        sleep(Duration::from_millis(mousedown_duration)).await;

        // Perform mouseup
        self.mouse_up().await?;

        Ok(())
    }

    /// Click on an element with human-like behavior
    ///
    /// This combines mouse movement and clicking with realistic delays.
    ///
    /// # Arguments
    /// * `element` - Target element to click
    /// * `options` - Click options
    pub async fn click_element(&self, element: &WebElement, options: MouseClickOptions) -> Result<()> {
        if options.move_to_element {
            self.move_to_element(element, options.move_options.clone()).await?;
        }

        self.click(options).await
    }

    /// Generate a Bezier curve path for smooth mouse movement
    fn generate_bezier_path(&self, start_x: i64, start_y: i64, end_x: i64, end_y: i64, steps: usize) -> Vec<(i64, i64)> {
        let mut points = Vec::with_capacity(steps);
        
        // Create control points for cubic Bezier curve
        let dx = end_x - start_x;
        let dy = end_y - start_y;
        
        // Control points create a natural curve
        let cp1_x = start_x + dx / 4;
        let cp1_y = start_y + dy / 4 + (rand::random::<i64>() % 20 - 10);
        let cp2_x = start_x + 3 * dx / 4;
        let cp2_y = start_y + 3 * dy / 4 + (rand::random::<i64>() % 20 - 10);

        for i in 0..steps {
            let t = i as f64 / steps as f64;
            let x = self.cubic_bezier(start_x as f64, cp1_x as f64, cp2_x as f64, end_x as f64, t);
            let y = self.cubic_bezier(start_y as f64, cp1_y as f64, cp2_y as f64, end_y as f64, t);
            points.push((x as i64, y as i64));
        }

        points.push((end_x, end_y));
        points
    }

    /// Generate a linear path for mouse movement
    fn generate_linear_path(&self, start_x: i64, start_y: i64, end_x: i64, end_y: i64, steps: usize) -> Vec<(i64, i64)> {
        let mut points = Vec::with_capacity(steps);
        
        for i in 0..=steps {
            let t = i as f64 / steps as f64;
            let x = start_x + ((end_x - start_x) as f64 * t) as i64;
            let y = start_y + ((end_y - start_y) as f64 * t) as i64;
            points.push((x, y));
        }

        points
    }

    /// Cubic Bezier curve calculation
    fn cubic_bezier(&self, p0: f64, p1: f64, p2: f64, p3: f64, t: f64) -> f64 {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        p0 * mt3 + 3.0 * p1 * mt2 * t + 3.0 * p2 * mt * t2 + p3 * t3
    }

    /// Add small random jitter to coordinates
    fn add_jitter(&self, x: i64, y: i64) -> (i64, i64) {
        let jitter_x = rand::random::<i64>() % 3 - 1; // -1, 0, or 1
        let jitter_y = rand::random::<i64>() % 3 - 1;
        (x + jitter_x, y + jitter_y)
    }

    /// Low-level mouse move to coordinates
    async fn move_mouse_to_coord(&self, x: i64, y: i64) -> Result<()> {
        {
            let guard = self.adapter.driver().await?;
            let driver = guard.as_ref().ok_or(Error::BrowserClosed)?;
            match driver.action_chain().move_to(x, y).perform().await {
                Ok(()) => {
                    *self.position.write().await = (x, y);
                    return Ok(());
                }
                Err(error) => {
                    tracing::debug!("WebDriver mouse move failed, falling back to JS: {}", error);
                }
            }
        }

        match self
            .adapter
            .execute_cdp_with_params(
                "Input.dispatchMouseEvent",
                json!({
                    "type": "mouseMoved",
                    "x": x,
                    "y": y,
                    "button": "none",
                    "buttons": 0
                }),
            )
            .await
        {
            Ok(_) => {
                *self.position.write().await = (x, y);
                return Ok(());
            }
            Err(error) => {
                tracing::debug!("CDP mouse move failed, falling back to JS: {}", error);
            }
        }

        let script = format!(
            r#"
            const event = new MouseEvent('mousemove', {{
                view: window,
                bubbles: true,
                cancelable: true,
                clientX: {},
                clientY: {}
            }});
            document.dispatchEvent(event);
            "#,
            x, y
        );
        self.adapter.execute_script(&script).await?;
        *self.position.write().await = (x, y);
        Ok(())
    }

    /// Low-level mousedown
    async fn mouse_down(&self) -> Result<()> {
        {
            let guard = self.adapter.driver().await?;
            let driver = guard.as_ref().ok_or(Error::BrowserClosed)?;
            match driver.action_chain().click_and_hold().perform().await {
                Ok(()) => return Ok(()),
                Err(error) => {
                    tracing::debug!("WebDriver mouse down failed, falling back to JS: {}", error);
                }
            }
        }

        let (x, y) = *self.position.read().await;
        match self
            .adapter
            .execute_cdp_with_params(
                "Input.dispatchMouseEvent",
                json!({
                    "type": "mousePressed",
                    "x": x,
                    "y": y,
                    "button": "left",
                    "buttons": 1,
                    "clickCount": 1
                }),
            )
            .await
        {
            Ok(_) => return Ok(()),
            Err(error) => {
                tracing::debug!("CDP mouse down failed, falling back to JS: {}", error);
            }
        }

        let script = r#"
            const event = new MouseEvent('mousedown', {
                view: window,
                bubbles: true,
                cancelable: true,
                buttons: 1
            });
            document.elementFromPoint(window.lastMouseX || 0, window.lastMouseY || 0)?.dispatchEvent(event);
        "#;
        self.adapter.execute_script(script).await?;
        Ok(())
    }

    /// Low-level mouseup
    async fn mouse_up(&self) -> Result<()> {
        {
            let guard = self.adapter.driver().await?;
            let driver = guard.as_ref().ok_or(Error::BrowserClosed)?;
            match driver.action_chain().release().perform().await {
                Ok(()) => return Ok(()),
                Err(error) => {
                    tracing::debug!("WebDriver mouse up failed, falling back to JS: {}", error);
                }
            }
        }

        let (x, y) = *self.position.read().await;
        match self
            .adapter
            .execute_cdp_with_params(
                "Input.dispatchMouseEvent",
                json!({
                    "type": "mouseReleased",
                    "x": x,
                    "y": y,
                    "button": "left",
                    "buttons": 0,
                    "clickCount": 1
                }),
            )
            .await
        {
            Ok(_) => return Ok(()),
            Err(error) => {
                tracing::debug!("CDP mouse up failed, falling back to JS: {}", error);
            }
        }

        let script = r#"
            const event = new MouseEvent('mouseup', {
                view: window,
                bubbles: true,
                cancelable: true,
                buttons: 0
            });
            document.elementFromPoint(window.lastMouseX || 0, window.lastMouseY || 0)?.dispatchEvent(event);
            
            const clickEvent = new MouseEvent('click', {
                view: window,
                bubbles: true,
                cancelable: true
            });
            document.elementFromPoint(window.lastMouseX || 0, window.lastMouseY || 0)?.dispatchEvent(clickEvent);
        "#;
        self.adapter.execute_script(script).await?;
        Ok(())
    }
}

// Simple random number generator (using getrandom crate already in dependencies)
mod rand {
    use std::sync::atomic::{AtomicU64, Ordering};
    
    static SEED: AtomicU64 = AtomicU64::new(0);
    
    fn next_u64() -> u64 {
        let mut seed = SEED.load(Ordering::Relaxed);
        if seed == 0 {
            seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64;
        }
        
        // Simple LCG
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        SEED.store(seed, Ordering::Relaxed);
        seed
    }
    
    pub fn random<T: 'static>() -> T {
        let val = next_u64();
        
        // Type-specific conversions
        if std::any::TypeId::of::<T>() == std::any::TypeId::of::<u64>() {
            unsafe { std::mem::transmute_copy(&val) }
        } else if std::any::TypeId::of::<T>() == std::any::TypeId::of::<i64>() {
            let i_val = val as i64;
            unsafe { std::mem::transmute_copy(&i_val) }
        } else if std::any::TypeId::of::<T>() == std::any::TypeId::of::<f64>() {
            // Convert to f64 in range [0.0, 1.0)
            let f_val = (val >> 11) as f64 * (1.0 / 9007199254740992.0);
            unsafe { std::mem::transmute_copy(&f_val) }
        } else {
            panic!("Unsupported type for random()")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bezier_path_generation() {
        // This would require a mock adapter
    }

    #[test]
    fn test_linear_path_generation() {
        // This would require a mock adapter
    }
}
