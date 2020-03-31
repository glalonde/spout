
class grid:
  def __init__(self, width, height):
    self.width = width
    self.height = height
    self.cells = [[]]
    for y in xrange(height):
      for x in xrange(width):
        self.cells[y].append(False)
      self.cells.append([])

  def fill_rect(self, x, y, width, height):
    if (x + width > self.width) or (y + height > self.height):
      return "Too Big"
    for ny in xrange(height):
      for nx in xrange(width):
        self.cells[y + ny][x + nx] = True
  def contains_point(self, p):
    if (p.x >= self.width) or \
        (p.x < 0) or \
        (p.y >= self.height) or \
        (p.y < 0):
      return False
    return True

  def get(self, x, y):
    return self.cells[y][x]

  def __str__(self):
    block = ''
    for y in xrange(self.height - 1, -1, -1):
      line = ''
      for x in xrange(self.width):
        if self.cells[y][x]:
          line += '#'
        else:
          line += '_'
      block += line + '\n'
    return block
