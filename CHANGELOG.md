
# 0.1.1 (2025-7-10)
- `Courier<F,C,R>`: The generic parameter `C` refers to the element type inside it, and is no longer wrapped in `Option`. Changing from `Courier<F, (Option<P>,..), R>`  to `Courier<F, (P,..), R>`.
- Examples: Improved clarity and readability

# 0.1.0 (2025-7-9)
- Initial implementation of core functionality.
- Implemented basic concepts of task, queue, thread and pool.