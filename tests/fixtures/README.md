# Test Fixture Image Set

This directory contains test images used to verify the background removal pipeline across diverse photographic conditions.

## Image Categories to be Added by Owner
1. `product-01.jpg` .. `product-NN.jpg`: Easy product photos with distinct backgrounds and solid edges.
2. `person-01.jpg` .. `person-NN.jpg`: Portrait photos with fine hair detail against varied backgrounds.
3. `animal-01.jpg` .. `animal-NN.jpg`: Pets or animals with complex fur edges.
4. `glass-01.jpg` .. `glass-NN.jpg`: Transparent or reflective objects (bottles, glassware).
5. `lowcontrast-01.jpg` .. `lowcontrast-NN.jpg`: Foreground subjects with color similar to background.
6. `large-01.jpg` .. `large-NN.jpg`: High-resolution images (4K+) to test memory bounds and downscaling.
7. `corrupt-01.jpg` .. `corrupt-NN.jpg`: Truncated, malformed, or zero-byte files to verify error handling.

## Rules for Test Fixtures
- Naming convention: `<category>-<number>.<ext>` (e.g., `product-01.jpg`, `person-02.png`).
- Git size constraint: Only small files (<1MB each) may be committed to git.
- Quality expectations: An accompanying markdown file or sidecar note should document expected quality for each image.
- Note: No test images are generated or committed during initial scaffolding.
