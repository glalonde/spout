from grid import grid
from point import point
import math

fuzziness = .00001

# Gets point of intersection between
# line (p1, p2) and (p3, p4)
def get_intersection(p1, p2, p3, p4, x12=None, y12=None):
  if not x12: x12 = (p1.x - p2.x)
  if not y12: y12 = (p1.y - p2.y)

  y34 = (p3.y - p4.y)
  x34 = (p3.x - p4.x)
  det = (x12 * y34) - (y12 * x34)
  if abs(det) < fuzziness:
    return None
  xy12 = (p1.x*p2.y - p1.y*p2.x)
  xy34 = (p3.x*p4.y - p3.y*p4.x)
  x = (xy12*x34 - x12*xy34)/float(det)
  y = (xy12*y34 - y12*xy34)/float(det)
  return point(x,y)

def get_ints_between(a, b):
  if a < b:
    p = math.ceil(a)
    while p < b:
      yield p
      p += 1
  else:
    p = math.floor(a)
    while p > b:
      yield p
      p -= 1

def get_next_int(iterable):
  try:
    return iterable.next()
  except StopIteration:
    return None

def l2_norm_squared(p1, p2):
  return (p1.x - p2.x)**2 + (p1.y - p1.y)**2

def get_collision(p1, p2, grid):
  if not grid.contains_point(p1):
    raise ValueError('p1 out of bounds')
  if not grid.contains_point(p2):
    raise ValueError('p2 out of bounds')
  if grid.get(int(p1.x), int(p1.y)):
    raise ValueError('p1 already inside')

  x12 = (p1.x - p2.x)
  y12 = (p1.y - p2.y)
  is_right = x12 < 0
  is_up = y12 < 0

  verticals = get_ints_between(p1.x, p2.x)
  horizontals = get_ints_between(p1.y, p2.y)

  def get_next_vertical_intersection():
    v = get_next_int(verticals)
    if not (v == None):
      return get_intersection(p1, p2, point(v, 0), point(v, 1), x12, y12)
    return None

  def get_next_horizontal_intersection():
    h = get_next_int(horizontals)
    if not (h == None):
      return get_intersection(p1, p2, point(0, h), point(1, h), x12, y12)
    return None

  def check_cell(point, is_vertical):
    if is_vertical:
      if is_right: 
        if grid.get(int(point.x), int(point.y)):
          print "hit: " + str(point)
          return (-1, 0)
      else:
        if grid.get(int(point.x) - 1, int(point.y)):
          print "hit: " + str(point)
          return (-1, 0)
    else:
      if is_up:
        if grid.get(int(point.x), int(point.y)):
          print "hit: " + str(point)
          return (0, -1)
      else:
        if grid.get(int(point.x), int(point.y) - 1):
          print "hit: " + str(point)
          return (0, -1)
    return None

  pv = get_next_vertical_intersection()
  ph = get_next_horizontal_intersection()
  done = False
  while not done:
    if not pv:
      if ph:
        done = check_cell(ph, False)
        ph = get_next_horizontal_intersection()
      else:
        done = True
    elif not ph:
      done = check_cell(pv, True)
      pv = get_next_vertical_intersection()
    else:
      dv = l2_norm_squared(p1, pv)
      dh = l2_norm_squared(p1, ph)
      if dh > dv:
        done = check_cell(pv, True)
        pv = get_next_vertical_intersection()
      else:
        done = check_cell(ph, False)
        ph = get_next_horizontal_intersection()

g = grid(5,5)
g.fill_rect(1,1,2,2)
print g

p1 = point(0,1.1)
p2 = point(2,1)

get_collision(p1, p2, g)
