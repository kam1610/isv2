## Node class ##############################################
class Node
  @type        = "header";
  @img         = "";
  @bt          = "";
  @has_n_and_c = "";
  @lbl         = "";
  @text        = "";
  @vertical    = "true";
  ## initialize ############################################
  def initialize(type:"Scene", img:"",      bt:"Child",     has_n_and_c:"None",
                 lbl:"",       text:"text", vertical:false, name:"")
    @type        = type;
    @img         = img;
    @bt          = bt;
    @has_n_and_c = has_n_and_c;
    @lbl         = lbl;
    @text        = text;
    @vertical    = vertical;
    @name        = name;
  end
  ## type ##################################################
  def type()
    return @type;
  end
  ## has_n_and_c ###########################################
  def has_n_and_c()
    return @has_n_and_c;
  end
  ## set_has_n_and_c #######################################
  def set_has_n_and_c(has_n_and_c)
    @has_n_and_c = has_n_and_c;
  end
  ## text ##################################################
  def text()
    return @text;
  end
  ## append_text ###########################################
  def append_text(text)
    @text += text;
  end
  ## header ##################################################
  def header()
    header = <<-EOS
    {"param_ser":
       {"param":
          {"target_width":2560,
           "target_height":1440,
           "project_dir":"./",
           "project_file_name":"project_00.json",
           "export_dir":"rel",
           "bgimg_en":true}},
    "sn_ser":[
    EOS
    return header;
  end
  ## scene ###################################################
  def scene(img, bt, has_n_and_c) # bt:={Child,Neighbor}, has_n_and_c:={Child,Neighbor,Both,None}
    scene = <<-EOS
     {"value":{"type":"Scene","bgimg":"#{img}","bg_en":true,
               "bgcol":{"r":128,"g":128,"b":128},
               "crop":{"pos":{"x":0,"y":0},"dim":{"w":2400,"h":2400}},
               "crop_en":true,
               "lbl":null,
               "lbl_type":"None"},
      "bt":"#{bt}",
      "id":1,
      "has_n_and_c":"#{has_n_and_c}"}
    EOS
    return scene;
  end
  ## page ####################################################
  def page(bt, name)
    page = <<-EOS
     {"value":{"type":"Page","name":"#{name}"},
      "bt":"#{bt}",
      "id":1000,
      "has_n_and_c":"#{has_n_and_c}"}
    EOS
    return page;
  end
  ## mat #####################################################
  def mat(lbl, bt, has_n_and_c, text, vertical) # vertical:={true,false}
    lv = (vertical) ? "true" : "false";
    mat = <<-EOS
     {"value":{"type":"Mat",
               "col":{"r":105,"g":124,"b":144},
               "pos":{"x":227,"y":196},
               "dim":{"w":127,"h":181},
               "r":8,"a":134,"src":null,
               "lbl":"#{lbl}",
               "lbl_type":"RefNoRect",
               "name":"mat",
               "font_col":{"r":0,"g":0,"b":0},
               "font_a":162,
               "font_weight":"Bold",
               "font_col_2":{"r":255,"g":255,"b":255},
               "font_a_2":177,
               "font_outl_2":4.0,
               "font_weight_2":"Normal",
               "font_size":22,
               "font_family":"Rounded M+ 1m",
               "line_spacing":0.8,"vertical":#{lv},
               "text":"#{text}",
               "bgimg":null,"bg_en":false,"text_pos":{"x":0,"y":0}},
      "bt":"#{bt}",
      "id":1001,
      "has_n_and_c":"#{has_n_and_c}"}
    EOS
    return mat;
  end
  ## footer ##################################################
  def footer()
    footer = "]}\n";
    return footer;
  end
  ## print #################################################
  def dump()
    case @type
    when "Scene"
      print( scene(@img, @bt, @has_n_and_c).chomp() );
    when "Page"
      print( page(@bt, @name).chomp() );
    when "Mat"
      print( mat(@lbl, @bt, @has_n_and_c, @text, @vertical).chomp() );
    when "Header"
      print( header().chomp() );
    when "Footer"
      print( footer().chomp() );
    else
      printf("unknown type");
      exit(1);
    end
  end
end
## search_prev_node ########################################
def search_prev_node(nodes, types, ix)
  while(0 <= ix)
    t = types.index(nodes[ix].type);
    if( t != nil )
      return [ types[t], ix ];
    end
    ix-= 1;
  end
  return ["None", -1];
end
## parse_file ##############################################
def parse_file(file, nodes, lbl, v, name="", include_stack=[])
  file = File.expand_path(file)

  if include_stack.include?(file)
    chain = (include_stack + [file]).join(" -> ")
    raise "recursive include detected: #{chain}"
  end

  include_stack.push(file)

  begin
    File.foreach(file).with_index(1) do |line, lineno|
      l = line.strip
      ix = nodes.length

      # check include directive before normal '# ...'
      if m = l.match(/^#\s+include\s+(.+?)\s*$/)
        include_file = m[1].strip
        include_path = File.expand_path(include_file, File.dirname(file))
        name = parse_file(include_path, nodes, lbl, v, name, include_stack)

      elsif l =~ /^# +/
        # label node, do nothing

      elsif m = l.match(/^## +([^ ].+)/) # Scene #################
        scene_arg = m[1].strip
        img = scene_arg.split(',')[0].strip

        prev = search_prev_node(nodes, ["Scene"], ix-1)
        bt = "Child"
        pix = prev[1]
        if 0 <= pix
          bt = "Neighbor"
          if nodes[pix].has_n_and_c() == "None"
            nodes[pix].set_has_n_and_c("Neighbor")
          else
            nodes[pix].set_has_n_and_c("Both")
          end
        end

        node = Node.new(type:"Scene", bt:bt, img:img, has_n_and_c:"None")
        nodes.push(node)

        n = scene_arg.split(',')[1]
        name = (n != nil) ? n.strip : ""

      elsif l =~ /^### +/ # Page ##########################
        bt = "Child"
        prev = search_prev_node(nodes, ["Scene", "Page"], ix-1)
        pix = prev[1]
        ptype = prev[0]

        if 0 <= pix
          pnc = nodes[pix].has_n_and_c()
        end

        if (0 <= pix) && (ptype == "Page")
          bt = "Neighbor"
          if pnc == "None"
            pnc = "Neighbor"
          elsif pnc == "Child"
            pnc = "Both"
          end
          nodes[pix].set_has_n_and_c(pnc)

        elsif (0 <= pix) && (ptype == "Scene")
          bt = "Child"
          if pnc == "None"
            pnc = "Child"
          elsif pnc == "Child"
            pnc = "Both"
          else
            printf("unexpected branch(Page) file=%s line=%d ix=%d\n", file, lineno, ix)
            exit(1)
          end
          nodes[pix].set_has_n_and_c(pnc)

        else
          printf("orphaned page file=%s line=%d (ix=%d)\n", file, lineno, ix)
          exit(1)
        end

        node = Node.new(type:"Page", bt:bt, has_n_and_c:"None", name:name)
        nodes.push(node)

      elsif m = l.match(/^#### +([^\s].*)$/) # Mat #################
        bt = "Child"
        prev = search_prev_node(nodes, ["Page", "Mat"], ix-1)
        pix = prev[1]
        ptype = prev[0]

        if (0 <= pix) && (ptype == "Page")
          nodes[pix].set_has_n_and_c("Child")
        elsif (0 <= pix) && (ptype == "Mat")
          bt = "Neighbor"
          nodes[pix].set_has_n_and_c("Neighbor")
        else
          printf("unexpected branch(Mat) file=%s line=%d ix=%d\n", file, lineno, ix)
          exit(1)
        end

        args = m[1].split(',').map{|arg| arg.strip }
        llbl = (1 < args.length) ? args[1] : lbl
        lv   = (2 < args.length) ? (args[2] == "true") : v

        node = Node.new(type:"Mat", bt:bt, lbl:llbl, text:"", vertical:lv)
        nodes.push(node)

      elsif l.empty? # empty line
        # empty line is ignored

      else
        if nodes.empty?
          printf("text without Mat node file=%s line=%d: %s\n", file, lineno, l)
          exit(1)
        end

        if nodes[-1].text() != ""
          l += "\\n"
        end
        nodes[-1].append_text(l)
      end
    end
  ensure
    include_stack.pop
  end

  return name
end

## dump_nodes ##############################################
def dump_nodes(nodes)
  h = Node.new(type:"Header")
  f = Node.new(type:"Footer")

  h.dump(); print("\n")
  nodes.each_with_index do |node, i|
    node.dump()
    print(",\n") if i < nodes.size - 1
  end
  print("\n")
  f.dump(); print("\n")
end

## main ####################################################
def main(file, lbl, v)
  nodes = []
  parse_file(file, nodes, lbl, v)
  dump_nodes(nodes)
end
############################################################
if( ARGV.length() == 0 )
  printf("usage ruby convert.rb input.md [default mat label] [default vertical true|false]");
  exit(0);
end


lbl = ( 1 < ARGV.length() ) ? ARGV[1].strip() : "";
v   = ( 2 < ARGV.length() ) ? (ARGV[2].strip() == "true") : true;

main(ARGV[0], lbl, v);
